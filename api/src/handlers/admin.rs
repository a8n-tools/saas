//! Admin handlers
//!
//! This module contains HTTP handlers for admin management endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::middleware::AdminUser;
use crate::models::{SubscriptionStatus, UserResponse};
use crate::repositories::{
    ApplicationRepository, AuditLogRepository, SubscriptionRepository, UserRepository,
};
use crate::responses::{get_request_id, paginated, success, success_no_data};

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
