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
use crate::services::{AuthService, EmailService};
use crate::validation::validate_email;

/// Request body for changing password
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Request body for requesting email change
#[derive(Debug, Deserialize)]
pub struct RequestEmailChangeBody {
    pub new_email: String,
    pub current_password: Option<String>,
}

/// Request body for confirming email change
#[derive(Debug, Deserialize)]
pub struct ConfirmEmailChangeBody {
    pub token: String,
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
    email_service: web::Data<Arc<EmailService>>,
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

    // Send password changed notification email (in background, don't wait)
    let email = user.0.email.clone();
    let email_svc = email_service.get_ref().clone();
    tokio::spawn(async move {
        if let Err(e) = email_svc.send_password_changed(&email).await {
            tracing::error!(error = %e, email = %email, "Failed to send password changed email");
        }
    });

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

/// POST /v1/users/me/email
/// Request email change
pub async fn request_email_change(
    req: HttpRequest,
    user: AuthenticatedUser,
    auth_service: web::Data<Arc<AuthService>>,
    email_service: web::Data<Arc<EmailService>>,
    body: web::Json<RequestEmailChangeBody>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    // Validate email format
    validate_email(&body.new_email)?;

    let (old_email, token) = auth_service
        .request_email_change(
            user.0.sub,
            body.new_email.clone(),
            body.current_password.clone(),
            ip_address,
        )
        .await?;

    match token {
        Some(token) => {
            // Verified user: send verification email to new address
            let new_email = body.new_email.clone();
            let email_svc = email_service.get_ref().clone();
            tokio::spawn(async move {
                if let Err(e) = email_svc.send_email_change_verify(&new_email, &token).await {
                    tracing::error!(error = %e, email = %new_email, "Failed to send email change verification");
                }
            });

            Ok(success(
                serde_json::json!({ "message": "Verification email sent to your new address. Please check your inbox.", "requires_relogin": false }),
                request_id,
            ))
        }
        None => {
            // Unverified user: email changed immediately, sessions revoked
            Ok(success(
                serde_json::json!({ "message": "Email address updated successfully. Please log in with your new email.", "requires_relogin": true }),
                request_id,
            ))
        }
    }
}

/// POST /v1/users/me/email/confirm
/// Confirm email change (token-based, no auth required)
pub async fn confirm_email_change(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    email_service: web::Data<Arc<EmailService>>,
    body: web::Json<ConfirmEmailChangeBody>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    let (old_email, new_email) = auth_service
        .confirm_email_change(body.token.clone(), ip_address)
        .await?;

    // Send notification to old email (fire and forget)
    let email_svc = email_service.get_ref().clone();
    tokio::spawn(async move {
        if let Err(e) = email_svc.send_email_change_notification(&old_email, &new_email).await {
            tracing::error!(error = %e, email = %old_email, "Failed to send email change notification");
        }
    });

    Ok(success(
        serde_json::json!({ "message": "Email address updated successfully. Please log in with your new email." }),
        request_id,
    ))
}
