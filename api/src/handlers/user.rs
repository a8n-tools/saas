//! User handlers
//!
//! This module contains HTTP handlers for user management endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::errors::AppError;
use crate::middleware::{extract_client_ip, AuthenticatedUser};
use crate::models::UserResponse;
use crate::repositories::{TokenRepository, UserRepository};
use crate::responses::{get_request_id, success, success_no_data};
use crate::services::AuthService;

/// Request body for changing password
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// GET /v1/users/me
/// Get current user profile
pub async fn get_current_user(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Get fresh user data from database
    let user = UserRepository::find_by_id(&pool, user.0.sub)
        .await?
        .ok_or(AppError::not_found("User"))?;

    Ok(success(UserResponse::from(user), request_id))
}

/// PUT /v1/users/me/password
/// Change current user's password
pub async fn change_password(
    req: HttpRequest,
    user: AuthenticatedUser,
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    auth_service
        .change_password(
            user.0.sub,
            body.current_password.clone(),
            body.new_password.clone(),
            ip_address,
        )
        .await?;

    Ok(success_no_data(request_id))
}

/// GET /v1/users/me/sessions
/// List active sessions for current user
pub async fn list_sessions(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let tokens = TokenRepository::find_active_refresh_tokens_for_user(&pool, user.0.sub).await?;

    // Map to response format (hide sensitive fields)
    let sessions: Vec<_> = tokens
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "device_info": t.device_info,
                "ip_address": t.ip_address.map(|ip| ip.to_string()),
                "created_at": t.created_at,
                "last_used_at": t.last_used_at,
            })
        })
        .collect();

    Ok(success(serde_json::json!({ "sessions": sessions }), request_id))
}

/// DELETE /v1/users/me/sessions/{session_id}
/// Revoke a specific session
pub async fn revoke_session(
    req: HttpRequest,
    user: AuthenticatedUser,
    path: web::Path<uuid::Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let session_id = path.into_inner();

    // Find the token and verify it belongs to the user
    let token = TokenRepository::find_refresh_token_by_id(&pool, session_id)
        .await?
        .ok_or(AppError::not_found("Session"))?;

    if token.user_id != user.0.sub {
        return Err(AppError::Forbidden);
    }

    // Revoke the token
    TokenRepository::revoke_refresh_token(&pool, session_id).await?;

    Ok(success_no_data(request_id))
}
