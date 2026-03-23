//! Admin handlers
//!
//! This module contains HTTP handlers for admin management endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::config::Config;
use crate::errors::AppError;
use crate::middleware::{AdminUser, AuthenticatedUser};
use crate::models::{
    AuditAction, CreateAuditLog, CreateApplication, CreatePasswordResetToken, CreateRefreshToken,
    DeleteApplicationRequest, MembershipStatus, StripeConfigResponse, SwapApplicationOrderRequest,
    UpdateApplication, UserResponse,
};
use crate::models::stripe::encrypt_secret;
use crate::repositories::{
    ApplicationRepository, AuditLogRepository, InviteRepository, MembershipRepository,
    NotificationRepository, StripeConfigRepository, TokenRepository, UserRepository,
};
use crate::responses::{get_request_id, created, paginated, success, success_no_data};
use crate::services::{AuthService, EmailService, JwtService, PasswordService, TotpService, WebhookService};
use crate::validation;

// =============================================================================
// User Management
// =============================================================================

/// Query parameters for listing users
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub search: Option<String>,
    pub status: Option<String>,
}

/// GET /v1/admin/users
/// List all users with pagination
pub async fn list_users(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<ListUsersQuery>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let status_filter = query.status.as_ref().map(|s| MembershipStatus::from(s.as_str()));

    let (users, total) = UserRepository::list_paginated(
        &pool,
        page,
        per_page,
        query.search.as_deref(),
        status_filter,
    )
    .await?;

    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(paginated(user_responses, total, page, per_page, request_id))
}

/// GET /v1/admin/users/{user_id}
/// Get a specific user
pub async fn get_user(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let user_id = path.into_inner();

    let user = UserRepository::find_by_id(&pool, user_id)
        .await?
        .ok_or(AppError::not_found("User"))?;

    Ok(success(UserResponse::from(user), request_id))
}

/// Request body for activating/deactivating user
#[derive(Debug, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub active: bool,
}

/// PUT /v1/admin/users/{user_id}/status
/// Activate or deactivate a user
pub async fn update_user_status(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateUserStatusRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let user_id = path.into_inner();

    if body.active {
        // Reactivate: clear deleted_at (would need new method)
        // For now, we can't reactivate soft-deleted users through this API
        return Err(AppError::validation(
            "active",
            "Cannot reactivate deleted users through this endpoint",
        ));
    } else {
        let target_user = UserRepository::find_by_id(&pool, user_id)
            .await?
            .ok_or(AppError::not_found("User"))?;

        UserRepository::soft_delete(&pool, user_id).await?;

        let audit_log = CreateAuditLog::new(AuditAction::AdminUserDeactivated)
            .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
            .with_resource("user", user_id)
            .with_metadata(serde_json::json!({
                "target_email": target_user.email,
            }));
        AuditLogRepository::create(&pool, audit_log).await?;
    }

    Ok(success_no_data(request_id))
}

/// DELETE /v1/admin/users/{user_id}
/// Delete a user (soft delete)
pub async fn delete_user(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let user_id = path.into_inner();

    // Prevent self-deletion
    if admin.0.sub == user_id {
        return Err(AppError::validation("user_id", "Cannot delete your own account"));
    }

    // Check if user exists
    let target_user = UserRepository::find_by_id(&pool, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

    // Prevent deleting other admins (optional safety measure)
    if target_user.role == "admin" {
        return Err(AppError::validation("user_id", "Cannot delete admin users"));
    }

    UserRepository::soft_delete(&pool, user_id).await?;

    tracing::info!(
        admin_id = %admin.0.sub,
        deleted_user_id = %user_id,
        deleted_user_email = %target_user.email,
        "Admin deleted user"
    );

    let audit_log = CreateAuditLog::new(AuditAction::AdminUserDeleted)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_resource("user", user_id)
        .with_metadata(serde_json::json!({
            "target_email": target_user.email,
            "target_role": target_user.role,
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success_no_data(request_id))
}

/// Request body for updating user role
#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

/// PUT /v1/admin/users/{user_id}/role
/// Change a user's role
pub async fn update_user_role(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateUserRoleRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let user_id = path.into_inner();

    // Validate role
    let valid_roles = ["subscriber", "admin"];
    if !valid_roles.contains(&body.role.as_str()) {
        return Err(AppError::validation("role", "Invalid role. Must be 'subscriber' or 'admin'"));
    }

    // Prevent changing own role
    if admin.0.sub == user_id {
        return Err(AppError::validation("user_id", "Cannot change your own role"));
    }

    let target_user = UserRepository::find_by_id(&pool, user_id)
        .await?
        .ok_or(AppError::not_found("User"))?;
    let old_role = target_user.role.clone();

    let updated_user = UserRepository::update_role(&pool, user_id, &body.role).await?;

    tracing::info!(
        admin_id = %admin.0.sub,
        target_user_id = %user_id,
        new_role = %body.role,
        "Admin changed user role"
    );

    let audit_log = CreateAuditLog::new(AuditAction::AdminUserRoleChanged)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_resource("user", user_id)
        .with_old_values(serde_json::json!({ "role": old_role }))
        .with_new_values(serde_json::json!({ "role": &body.role }))
        .with_metadata(serde_json::json!({
            "target_email": target_user.email,
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success(UserResponse::from(updated_user), request_id))
}

// =============================================================================
// Membership Management
// =============================================================================

/// Request body for granting membership
#[derive(Debug, Deserialize)]
pub struct GrantMembershipRequest {
    pub user_id: uuid::Uuid,
    pub price_locked: Option<bool>,
    pub locked_price_amount: Option<i32>,
}

/// POST /v1/admin/memberships/grant
/// Grant a membership to a user
pub async fn grant_membership(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    body: web::Json<GrantMembershipRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Update user membership status
    UserRepository::update_membership_status(pool.get_ref(), body.user_id, MembershipStatus::Active)
        .await?;

    // Lock price if requested
    let price_locked = body.price_locked.unwrap_or(false);
    let locked_amount = body.locked_price_amount.unwrap_or(300);
    if price_locked {
        UserRepository::lock_price(pool.get_ref(), body.user_id, "price_admin_grant", locked_amount).await?;
    }

    let audit_log = CreateAuditLog::new(AuditAction::AdminMembershipGranted)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_resource("user", body.user_id)
        .with_metadata(serde_json::json!({
            "price_locked": price_locked,
            "locked_price_amount": locked_amount,
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success_no_data(request_id))
}

/// POST /v1/admin/memberships/revoke
/// Revoke a membership from a user
pub async fn revoke_membership(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    body: web::Json<GrantMembershipRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    UserRepository::update_membership_status(pool.get_ref(), body.user_id, MembershipStatus::Canceled)
        .await?;

    // Clear any grace period
    UserRepository::clear_grace_period(pool.get_ref(), body.user_id).await?;

    let audit_log = CreateAuditLog::new(AuditAction::AdminMembershipRevoked)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_resource("user", body.user_id);
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success_no_data(request_id))
}

/// Query parameters for listing memberships
#[derive(Debug, Deserialize)]
pub struct ListMembershipsQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<String>,
}

/// GET /v1/admin/memberships
/// List all memberships with pagination
pub async fn list_memberships(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<ListMembershipsQuery>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let (memberships, total) =
        MembershipRepository::list_paginated(&pool, page, per_page, query.status.as_deref())
            .await?;

    Ok(paginated(memberships, total, page, per_page, request_id))
}

// =============================================================================
// Application Management
// =============================================================================

/// GET /v1/admin/applications
/// List all applications (including inactive)
pub async fn list_all_applications(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let apps = ApplicationRepository::list_all(&pool).await?;

    Ok(success(serde_json::json!({ "applications": apps }), request_id))
}

/// PUT /v1/admin/applications/{app_id}/swap-order
/// Swap the sort order of two applications
pub async fn swap_application_order(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<SwapApplicationOrderRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let app_id = path.into_inner();

    ApplicationRepository::swap_sort_order(&pool, app_id, body.target_app_id).await?;

    let apps = ApplicationRepository::list_all(&pool).await?;

    Ok(success(serde_json::json!({ "applications": apps }), request_id))
}

/// PUT /v1/admin/applications/{app_id}
/// Update an application
pub async fn update_application(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateApplication>,
    webhook_service: web::Data<Arc<WebhookService>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let app_id = path.into_inner();

    let old_app = ApplicationRepository::find_by_id(&pool, app_id)
        .await?
        .ok_or(AppError::not_found("Application"))?;

    let app = ApplicationRepository::update(&pool, app_id, &body).await?;

    // Notify child app if maintenance mode or active status changed
    let maintenance_changed = old_app.maintenance_mode != app.maintenance_mode;
    let active_changed = old_app.is_active != app.is_active;
    if maintenance_changed || active_changed {
        let ws = webhook_service.into_inner();
        let app_clone = app.clone();
        actix_web::rt::spawn(async move {
            if maintenance_changed {
                ws.notify_maintenance_change(&app_clone).await;
            }
            if active_changed {
                ws.notify_active_change(&app_clone).await;
            }
        });
    }

    // Audit log for all application updates
    let audit_log = CreateAuditLog::new(AuditAction::ApplicationUpdated)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_resource("application", app_id)
        .with_old_values(serde_json::json!({
            "name": old_app.name,
            "is_active": old_app.is_active,
            "maintenance_mode": old_app.maintenance_mode,
        }))
        .with_new_values(serde_json::json!({
            "name": app.name,
            "is_active": app.is_active,
            "maintenance_mode": app.maintenance_mode,
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    // Additional specific log when maintenance mode changes
    if maintenance_changed {
        let maintenance_log = CreateAuditLog::new(AuditAction::ApplicationMaintenanceToggled)
            .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
            .with_resource("application", app_id)
            .with_metadata(serde_json::json!({
                "application_name": app.name,
                "maintenance_mode": app.maintenance_mode,
            }));
        AuditLogRepository::create(&pool, maintenance_log).await?;
    }

    Ok(success(app, request_id))
}

/// POST /v1/admin/applications
/// Create a new application
pub async fn create_application(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    body: web::Json<CreateApplication>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Validate required fields
    if body.name.trim().is_empty() {
        return Err(AppError::validation("name", "Name is required"));
    }
    if body.slug.trim().is_empty() {
        return Err(AppError::validation("slug", "Slug is required"));
    }
    if body.display_name.trim().is_empty() {
        return Err(AppError::validation("display_name", "Display name is required"));
    }
    if body.container_name.trim().is_empty() {
        return Err(AppError::validation("container_name", "Container name is required"));
    }

    // Validate slug format
    validation::validate_slug(&body.slug).map_err(|_| {
        AppError::validation("slug", "Slug must contain only lowercase letters, numbers, and hyphens")
    })?;

    // Check slug uniqueness
    if ApplicationRepository::find_by_slug(&pool, &body.slug)
        .await?
        .is_some()
    {
        return Err(AppError::conflict("An application with this slug already exists"));
    }

    let app = ApplicationRepository::create(&pool, &body).await?;

    // Audit log
    let audit_log = CreateAuditLog::new(AuditAction::ApplicationCreated)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_resource("application", app.id)
        .with_metadata(serde_json::json!({
            "application_name": app.name,
            "application_slug": app.slug,
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(created(app, request_id))
}

/// DELETE /v1/admin/applications/{app_id}
/// Delete an application (requires password + 2FA)
pub async fn delete_application(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<DeleteApplicationRequest>,
    totp_service: web::Data<Arc<TotpService>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let app_id = path.into_inner();

    // Look up the admin user to get password hash
    let admin_user = UserRepository::find_by_id(&pool, admin.0.sub)
        .await?
        .ok_or(AppError::not_found("User"))?;

    // Verify password
    let password_service = PasswordService::new();
    let password_hash = admin_user
        .password_hash
        .as_deref()
        .ok_or_else(|| AppError::validation("password", "Account has no password set"))?;
    if !password_service.verify(&body.password, password_hash)? {
        return Err(AppError::validation("password", "Invalid password"));
    }

    // Verify TOTP code (2FA must be enabled)
    let totp_valid = totp_service
        .verify_code(admin.0.sub, &body.totp_code)
        .await
        .map_err(|_| AppError::validation("totp_code", "2FA must be enabled to delete applications"))?;
    if !totp_valid {
        return Err(AppError::validation("totp_code", "Invalid 2FA code"));
    }

    // Find the application (for audit metadata)
    let app = ApplicationRepository::find_by_id(&pool, app_id)
        .await?
        .ok_or(AppError::not_found("Application"))?;

    // Delete
    ApplicationRepository::delete(&pool, app_id).await?;

    // Audit log
    let audit_log = CreateAuditLog::new(AuditAction::ApplicationDeleted)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_resource("application", app_id)
        .with_metadata(serde_json::json!({
            "application_name": app.name,
            "application_slug": app.slug,
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success_no_data(request_id))
}

// =============================================================================
// Audit Logs
// =============================================================================

/// Query parameters for listing audit logs
#[derive(Debug, Deserialize)]
pub struct ListAuditLogsQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub user_id: Option<uuid::Uuid>,
    pub action: Option<String>,
}

/// GET /v1/admin/audit-logs
/// List audit logs with pagination
pub async fn list_audit_logs(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<ListAuditLogsQuery>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(100);

    let (logs, total) = AuditLogRepository::list_paginated(
        &pool,
        page,
        per_page,
        query.user_id,
        query.action.as_deref(),
        false, // admin_only
        None,  // start_date
        None,  // end_date
    )
    .await?;

    Ok(paginated(logs, total, page, per_page, request_id))
}

// =============================================================================
// Dashboard Stats
// =============================================================================

/// Dashboard statistics response
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub active_members: i64,
    pub past_due_members: i64,
    pub grace_period_members: i64,
    pub total_applications: i64,
    pub active_applications: i64,
}

/// GET /v1/admin/stats
/// Get dashboard statistics
pub async fn get_dashboard_stats(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Get user counts by status
    let total_users: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(pool.get_ref())
            .await?;

    let active_members: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE subscription_status = 'active' AND deleted_at IS NULL",
    )
    .fetch_one(pool.get_ref())
    .await?;

    let past_due_members: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE subscription_status = 'past_due' AND deleted_at IS NULL",
    )
    .fetch_one(pool.get_ref())
    .await?;

    let grace_period_members: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE subscription_status = 'grace_period' AND deleted_at IS NULL",
    )
    .fetch_one(pool.get_ref())
    .await?;

    let total_applications: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM applications")
            .fetch_one(pool.get_ref())
            .await?;

    let active_applications: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM applications WHERE is_active = TRUE")
            .fetch_one(pool.get_ref())
            .await?;

    let stats = DashboardStats {
        total_users: total_users.0,
        active_members: active_members.0,
        past_due_members: past_due_members.0,
        grace_period_members: grace_period_members.0,
        total_applications: total_applications.0,
        active_applications: active_applications.0,
    };

    Ok(success(stats, request_id))
}

// =============================================================================
// User Actions (Reset Password, Impersonate)
// =============================================================================

/// POST /v1/admin/users/{user_id}/reset-password
/// Trigger a password reset email for a user
pub async fn admin_reset_password(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    jwt_service: web::Data<Arc<JwtService>>,
    email_service: web::Data<Arc<EmailService>>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let user_id = path.into_inner();
    let admin_user_id = admin.0.sub;

    // Find the user
    let user = UserRepository::find_by_id(&pool, user_id)
        .await?
        .ok_or(AppError::not_found("User"))?;

    // Generate password reset token
    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = jwt_service.hash_token(&raw_token);
    let expires_at = Utc::now() + Duration::hours(1);

    TokenRepository::create_password_reset_token(
        &pool,
        CreatePasswordResetToken {
            user_id,
            token_hash,
            expires_at,
            ip_address: None,
        },
    )
    .await?;

    // Send password reset email
    email_service.send_password_reset(&user.email, &raw_token).await?;

    // Log admin action
    let audit_log = CreateAuditLog::new(AuditAction::AdminPasswordReset)
        .with_actor(admin_user_id, &admin.0.email, &admin.0.role)
        .with_resource("user", user_id)
        .with_metadata(serde_json::json!({
            "target_user_id": user_id,
            "target_email": user.email
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success_no_data(request_id))
}

/// POST /v1/admin/users/{user_id}/impersonate
/// Generate tokens to impersonate a user
pub async fn impersonate_user(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    jwt_service: web::Data<Arc<JwtService>>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let target_user_id = path.into_inner();
    let admin_user_id = admin.0.sub;

    // Prevent self-impersonation
    if admin_user_id == target_user_id {
        return Err(AppError::validation("user_id", "Cannot impersonate yourself"));
    }

    // Find the target user
    let target_user = UserRepository::find_by_id(&pool, target_user_id)
        .await?
        .ok_or(AppError::not_found("User"))?;

    // Generate access token for target user
    let access_token = jwt_service.create_access_token(&target_user)?;

    // Generate refresh token
    let (refresh_token, token_hash) = jwt_service.create_refresh_token(target_user.id)?;
    let expires_at = Utc::now() + Duration::days(30);

    TokenRepository::create_refresh_token(
        &pool,
        CreateRefreshToken {
            user_id: target_user.id,
            token_hash,
            device_info: Some("Admin impersonation".to_string()),
            ip_address: None,
            expires_at,
        },
    )
    .await?;

    // Log admin action
    let audit_log = CreateAuditLog::new(AuditAction::AdminUserImpersonated)
        .with_actor(admin_user_id, &admin.0.email, &admin.0.role)
        .with_resource("user", target_user_id)
        .with_metadata(serde_json::json!({
            "target_user_id": target_user_id,
            "target_email": target_user.email,
            "admin_id": admin_user_id
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success(
        serde_json::json!({
            "access_token": access_token,
            "refresh_token": refresh_token,
            "user": UserResponse::from(target_user)
        }),
        request_id,
    ))
}

// =============================================================================
// Notifications
// =============================================================================

/// Query parameters for listing notifications
#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub unread: Option<bool>,
}

/// GET /v1/admin/notifications
/// List admin notifications
pub async fn list_notifications(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<ListNotificationsQuery>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    if query.unread.unwrap_or(false) {
        let notifications = NotificationRepository::list_unread(&pool).await?;
        let total = notifications.len() as i64;
        return Ok(paginated(notifications, total, 1, 100, request_id));
    }

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let (notifications, total) =
        NotificationRepository::list_paginated(&pool, page, per_page).await?;

    Ok(paginated(notifications, total, page, per_page, request_id))
}

/// POST /v1/admin/notifications/{notification_id}/read
/// Mark a notification as read
pub async fn mark_notification_read(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let notification_id = path.into_inner();

    NotificationRepository::mark_as_read(&pool, notification_id, admin.0.sub).await?;

    Ok(success_no_data(request_id))
}

/// POST /v1/admin/notifications/read-all
/// Mark all notifications as read
pub async fn mark_all_notifications_read(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    NotificationRepository::mark_all_as_read(&pool, admin.0.sub).await?;

    Ok(success_no_data(request_id))
}

// =============================================================================
// Test Email
// =============================================================================

/// POST /v1/admin/test-email
/// Send a test welcome email to the authenticated user
pub async fn send_test_email(
    req: HttpRequest,
    user: AuthenticatedUser,
    email_service: web::Data<Arc<EmailService>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    email_service
        .send_welcome(&user.0.email, 300)
        .await?;

    tracing::info!(email = %user.0.email, "Test email sent");

    Ok(success_no_data(request_id))
}

// =============================================================================
// System Health
// =============================================================================

/// System health response
#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub status: String,
    pub database: HealthStatus,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Health status for a component
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

// =============================================================================
// Admin Invites
// =============================================================================

/// Request body for creating an admin invite
#[derive(Debug, Deserialize)]
pub struct CreateAdminInviteRequest {
    pub email: String,
}

/// Query parameters for listing admin invites
#[derive(Debug, Deserialize)]
pub struct ListAdminInvitesQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// POST /v1/admin/invites
/// Create an admin invite and send email
pub async fn create_admin_invite(
    req: HttpRequest,
    admin: AdminUser,
    auth_service: web::Data<Arc<AuthService>>,
    email_service: web::Data<Arc<EmailService>>,
    body: web::Json<CreateAdminInviteRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = crate::middleware::extract_client_ip(&req);

    // Validate email format
    crate::validation::validate_email(&body.email)?;

    let token = auth_service
        .create_admin_invite(
            body.email.clone(),
            admin.0.sub,
            &admin.0.email,
            &admin.0.role,
            ip_address,
        )
        .await?;

    // Send invite email (in background)
    let email = body.email.clone();
    let email_svc = email_service.get_ref().clone();
    tokio::spawn(async move {
        if let Err(e) = email_svc.send_admin_invite(&email, &token).await {
            tracing::error!(error = %e, email = %email, "Failed to send admin invite email");
        }
    });

    Ok(created(serde_json::json!({ "email": body.email }), request_id))
}

/// GET /v1/admin/invites
/// List admin invites with pagination
pub async fn list_admin_invites(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<ListAdminInvitesQuery>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let (invites, total) = InviteRepository::list_all(&pool, page, per_page).await?;

    Ok(paginated(invites, total, page, per_page, request_id))
}

/// DELETE /v1/admin/invites/{invite_id}
/// Revoke a pending admin invite
pub async fn revoke_admin_invite(
    req: HttpRequest,
    admin: AdminUser,
    auth_service: web::Data<Arc<AuthService>>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let invite_id = path.into_inner();

    auth_service
        .revoke_admin_invite(invite_id, admin.0.sub, &admin.0.email, &admin.0.role)
        .await?;

    Ok(success_no_data(request_id))
}

/// GET /v1/admin/health
/// Get system health status
pub async fn get_system_health(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Check database health
    let db_start = std::time::Instant::now();
    let db_health = match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HealthStatus {
            status: "healthy".to_string(),
            latency_ms: Some(db_start.elapsed().as_millis() as u64),
            message: None,
        },
        Err(e) => HealthStatus {
            status: "unhealthy".to_string(),
            latency_ms: None,
            message: Some(e.to_string()),
        },
    };

    // Get database stats
    let db_stats: Option<(i64, i64, i64)> = sqlx::query_as(
        r#"
        SELECT
            (SELECT COUNT(*) FROM users WHERE deleted_at IS NULL) as users,
            (SELECT COUNT(*) FROM users WHERE subscription_status = 'active') as active_subs,
            (SELECT COUNT(*) FROM audit_logs WHERE created_at > NOW() - INTERVAL '1 hour') as recent_logs
        "#
    )
    .fetch_optional(pool.get_ref())
    .await
    .ok()
    .flatten();

    let overall_status = if db_health.status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    let health = SystemHealth {
        status: overall_status.to_string(),
        database: db_health,
        uptime_seconds: 0, // Would need to track startup time
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let mut response = serde_json::json!({
        "health": health,
    });

    if let Some((users, active_members, recent_logs)) = db_stats {
        response["stats"] = serde_json::json!({
            "total_users": users,
            "active_members": active_members,
            "audit_logs_last_hour": recent_logs
        });
    }

    Ok(success(response, request_id))
}

// =============================================================================
// Stripe Config
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateStripeConfigRequest {
    pub secret_key: Option<String>,
    pub webhook_secret: Option<String>,
    pub price_id_personal: Option<String>,
    pub price_id_business: Option<String>,
}

/// GET /v1/admin/stripe
/// Returns the current Stripe config with secrets masked.
/// Falls back to env vars if no DB config has been saved yet.
pub async fn get_stripe_config(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let db = StripeConfigRepository::get(&pool).await?;

    let response = if db.secret_key.is_some()
        || db.webhook_secret.is_some()
        || db.price_id_personal.is_some()
        || db.price_id_business.is_some()
    {
        StripeConfigResponse::from_db(&db, &config.stripe_encryption_key)?
    } else {
        StripeConfigResponse::from_env()
    };

    Ok(success(response, request_id))
}

/// PUT /v1/admin/stripe
/// Updates Stripe config. Only fields with a non-empty value are written; omitted or
/// empty-string fields leave the existing DB value unchanged.
pub async fn update_stripe_config(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    body: web::Json<UpdateStripeConfigRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let key = &config.stripe_encryption_key;

    // Treat empty strings the same as None — user left the field blank
    let secret_key_plain = body.secret_key.as_deref().filter(|s| !s.is_empty());
    let webhook_secret_plain = body.webhook_secret.as_deref().filter(|s| !s.is_empty());
    let price_id_personal = body.price_id_personal.as_deref().filter(|s| !s.is_empty());
    let price_id_business = body.price_id_business.as_deref().filter(|s| !s.is_empty());

    // Encrypt secrets before storing
    let (secret_key_enc, secret_key_nonce) = match secret_key_plain {
        Some(sk) => {
            let (ct, nonce) = encrypt_secret(key, sk)?;
            (Some(ct), Some(nonce))
        }
        None => (None, None),
    };
    let (webhook_secret_enc, webhook_secret_nonce) = match webhook_secret_plain {
        Some(ws) => {
            let (ct, nonce) = encrypt_secret(key, ws)?;
            (Some(ct), Some(nonce))
        }
        None => (None, None),
    };

    let updated = StripeConfigRepository::update(
        &pool,
        secret_key_enc,
        secret_key_nonce,
        webhook_secret_enc,
        webhook_secret_nonce,
        price_id_personal,
        price_id_business,
        admin.0.sub,
    )
    .await?;

    let audit_log = CreateAuditLog::new(AuditAction::AdminStripeConfigUpdated)
        .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
        .with_metadata(serde_json::json!({
            "fields_updated": {
                "secret_key": secret_key_plain.is_some(),
                "webhook_secret": webhook_secret_plain.is_some(),
                "price_id_personal": price_id_personal.is_some(),
                "price_id_business": price_id_business.is_some(),
            }
        }));
    AuditLogRepository::create(&pool, audit_log).await?;

    Ok(success(StripeConfigResponse::from_db(&updated, key)?, request_id))
}

// =============================================================================
// Subscription Management
// =============================================================================

/// POST /v1/admin/users/{user_id}/lifetime
/// Grant lifetime membership to a user
pub async fn grant_lifetime_membership(
    req: HttpRequest,
    admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let user_id = path.into_inner();

    let user = UserRepository::grant_lifetime_membership(&pool, user_id, admin.0.sub).await?;

    AuditLogRepository::create(
        &pool,
        CreateAuditLog::new(AuditAction::AdminMembershipGranted)
            .with_actor(admin.0.sub, &admin.0.email, &admin.0.role)
            .with_resource("user", user_id)
            .with_metadata(serde_json::json!({
                "tier": "lifetime",
                "target_email": user.email,
            })),
    )
    .await?;

    Ok(success(UserResponse::from(user), request_id))
}
