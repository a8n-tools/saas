//! Admin handlers
//!
//! This module contains HTTP handlers for admin management endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::errors::AppError;
use crate::middleware::AdminUser;
use crate::models::{
    AuditAction, CreateAuditLog, CreatePasswordResetToken, CreateRefreshToken, SubscriptionStatus,
    UserResponse,
};
use crate::repositories::{
    ApplicationRepository, AuditLogRepository, NotificationRepository, SubscriptionRepository,
    TokenRepository, UserRepository,
};
use crate::responses::{get_request_id, paginated, success, success_no_data};
use crate::services::{EmailService, JwtService};

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
    let status_filter = query.status.as_ref().map(|s| SubscriptionStatus::from(s.as_str()));

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
    _admin: AdminUser,
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
        UserRepository::soft_delete(&pool, user_id).await?;
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

    let updated_user = UserRepository::update_role(&pool, user_id, &body.role).await?;

    tracing::info!(
        admin_id = %admin.0.sub,
        target_user_id = %user_id,
        new_role = %body.role,
        "Admin changed user role"
    );

    Ok(success(UserResponse::from(updated_user), request_id))
}

// =============================================================================
// Subscription Management
// =============================================================================

/// Request body for granting subscription
#[derive(Debug, Deserialize)]
pub struct GrantSubscriptionRequest {
    pub user_id: uuid::Uuid,
    pub price_locked: Option<bool>,
    pub locked_price_amount: Option<i32>,
}

/// POST /v1/admin/subscriptions/grant
/// Grant a subscription to a user
pub async fn grant_subscription(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    body: web::Json<GrantSubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Update user subscription status
    UserRepository::update_subscription_status(&pool, body.user_id, SubscriptionStatus::Active)
        .await?;

    // Lock price if requested
    if body.price_locked.unwrap_or(false) {
        let amount = body.locked_price_amount.unwrap_or(300);
        UserRepository::lock_price(&pool, body.user_id, "price_admin_grant", amount).await?;
    }

    Ok(success_no_data(request_id))
}

/// POST /v1/admin/subscriptions/revoke
/// Revoke a subscription from a user
pub async fn revoke_subscription(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    body: web::Json<GrantSubscriptionRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    UserRepository::update_subscription_status(&pool, body.user_id, SubscriptionStatus::Canceled)
        .await?;

    // Clear any grace period
    UserRepository::clear_grace_period(&pool, body.user_id).await?;

    Ok(success_no_data(request_id))
}

/// Query parameters for listing subscriptions
#[derive(Debug, Deserialize)]
pub struct ListSubscriptionsQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<String>,
}

/// GET /v1/admin/subscriptions
/// List all subscriptions with pagination
pub async fn list_subscriptions(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    query: web::Query<ListSubscriptionsQuery>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let (subscriptions, total) =
        SubscriptionRepository::list_paginated(&pool, page, per_page, query.status.as_deref())
            .await?;

    Ok(paginated(subscriptions, total, page, per_page, request_id))
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

/// Request body for updating application
#[derive(Debug, Deserialize)]
pub struct UpdateApplicationRequest {
    pub is_active: Option<bool>,
    pub maintenance_mode: Option<bool>,
    pub maintenance_message: Option<String>,
    pub version: Option<String>,
}

/// PUT /v1/admin/applications/{app_id}
/// Update an application
pub async fn update_application(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateApplicationRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let app_id = path.into_inner();

    // Verify app exists
    ApplicationRepository::find_by_id(&pool, app_id)
        .await?
        .ok_or(AppError::not_found("Application"))?;

    if let Some(active) = body.is_active {
        ApplicationRepository::set_active(&pool, app_id, active).await?;
    }

    if let Some(maintenance) = body.maintenance_mode {
        ApplicationRepository::set_maintenance_mode(
            &pool,
            app_id,
            maintenance,
            body.maintenance_message.as_deref(),
        )
        .await?;
    }

    if let Some(ref version) = body.version {
        ApplicationRepository::update_version(&pool, app_id, version).await?;
    }

    // Get updated app
    let app = ApplicationRepository::find_by_id(&pool, app_id)
        .await?
        .ok_or(AppError::not_found("Application"))?;

    Ok(success(app, request_id))
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
    pub active_subscribers: i64,
    pub past_due_subscribers: i64,
    pub grace_period_subscribers: i64,
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

    let active_subscribers: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE subscription_status = 'active' AND deleted_at IS NULL",
    )
    .fetch_one(pool.get_ref())
    .await?;

    let past_due_subscribers: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE subscription_status = 'past_due' AND deleted_at IS NULL",
    )
    .fetch_one(pool.get_ref())
    .await?;

    let grace_period_subscribers: (i64,) = sqlx::query_as(
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
        active_subscribers: active_subscribers.0,
        past_due_subscribers: past_due_subscribers.0,
        grace_period_subscribers: grace_period_subscribers.0,
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

    if let Some((users, active_subs, recent_logs)) = db_stats {
        response["stats"] = serde_json::json!({
            "total_users": users,
            "active_subscribers": active_subs,
            "audit_logs_last_hour": recent_logs
        });
    }

    Ok(success(response, request_id))
}
