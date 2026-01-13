//! Authentication handlers
//!
//! This module contains HTTP handlers for authentication endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::errors::AppError;
use crate::middleware::{
    extract_client_ip, extract_device_info, AuthCookies, AuthenticatedUser,
};
use crate::models::UserResponse;
use crate::responses::{get_request_id, success};
use crate::services::AuthService;

/// Request body for user registration
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// Request body for login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

/// Request body for magic link request
#[derive(Debug, Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

/// Request body for magic link verification
#[derive(Debug, Deserialize)]
pub struct VerifyMagicLinkRequest {
    pub token: String,
}

/// Request body for password reset request
#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// Request body for password reset confirmation
#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmRequest {
    pub token: String,
    pub new_password: String,
}

/// Response for successful authentication
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub expires_in: i64,
}

/// POST /v1/auth/register
/// Register a new user
pub async fn register(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    // Validate email format
    crate::validation::validate_email(&body.email)?;

    let user = auth_service
        .register(body.email.clone(), body.password.clone(), ip_address)
        .await?;

    Ok(crate::responses::created(user, request_id))
}

/// POST /v1/auth/login
/// Login with email and password
pub async fn login(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<LoginRequest>,
    config: web::Data<crate::config::Config>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);
    let device_info = extract_device_info(&req);

    let (tokens, user) = auth_service
        .login(
            body.email.clone(),
            body.password.clone(),
            device_info,
            ip_address,
        )
        .await?;

    let secure = config.is_production();
    let cookie_domain = config.cookie_domain.as_deref();

    let response = AuthResponse {
        user,
        expires_in: tokens.expires_in,
    };

    Ok(HttpResponse::Ok()
        .cookie(AuthCookies::access_token(&tokens.access_token, secure, cookie_domain))
        .cookie(AuthCookies::refresh_token(
            &tokens.refresh_token,
            secure,
            body.remember,
            cookie_domain,
        ))
        .json(crate::responses::ApiResponse {
            success: true,
            data: Some(response),
            meta: crate::responses::ResponseMeta::new(request_id),
        }))
}

/// POST /v1/auth/magic-link
/// Request a magic link for passwordless login
pub async fn request_magic_link(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    email_service: web::Data<Arc<crate::services::EmailService>>,
    body: web::Json<MagicLinkRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    // Validate email format
    crate::validation::validate_email(&body.email)?;

    // Generate magic link token
    let token = auth_service
        .request_magic_link(body.email.clone(), ip_address)
        .await?;

    // Send email (in background, don't wait)
    let email = body.email.clone();
    let email_svc = email_service.get_ref().clone();
    tokio::spawn(async move {
        if let Err(e) = email_svc.send_magic_link(&email, &token).await {
            tracing::error!(error = %e, email = %email, "Failed to send magic link email");
        }
    });

    // Always return success (don't reveal if email exists)
    Ok(HttpResponse::Accepted().json(crate::responses::ApiResponse::<()> {
        success: true,
        data: None,
        meta: crate::responses::ResponseMeta::new(request_id),
    }))
}

/// POST /v1/auth/magic-link/verify
/// Verify a magic link and login
pub async fn verify_magic_link(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<VerifyMagicLinkRequest>,
    config: web::Data<crate::config::Config>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);
    let device_info = extract_device_info(&req);

    let (tokens, user) = auth_service
        .verify_magic_link(body.token.clone(), device_info, ip_address)
        .await?;

    let secure = config.is_production();
    let cookie_domain = config.cookie_domain.as_deref();

    let response = AuthResponse {
        user,
        expires_in: tokens.expires_in,
    };

    Ok(HttpResponse::Ok()
        .cookie(AuthCookies::access_token(&tokens.access_token, secure, cookie_domain))
        .cookie(AuthCookies::refresh_token(&tokens.refresh_token, secure, true, cookie_domain))
        .json(crate::responses::ApiResponse {
            success: true,
            data: Some(response),
            meta: crate::responses::ResponseMeta::new(request_id),
        }))
}

/// POST /v1/auth/refresh
/// Refresh access token using refresh token
pub async fn refresh_token(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    config: web::Data<crate::config::Config>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);
    let device_info = extract_device_info(&req);

    // Get refresh token from cookie
    let refresh_token = req
        .cookie("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let tokens = auth_service
        .refresh_tokens(refresh_token, device_info, ip_address)
        .await?;

    let secure = config.is_production();
    let cookie_domain = config.cookie_domain.as_deref();

    Ok(HttpResponse::Ok()
        .cookie(AuthCookies::access_token(&tokens.access_token, secure, cookie_domain))
        .cookie(AuthCookies::refresh_token(&tokens.refresh_token, secure, true, cookie_domain))
        .json(crate::responses::ApiResponse {
            success: true,
            data: Some(serde_json::json!({ "expires_in": tokens.expires_in })),
            meta: crate::responses::ResponseMeta::new(request_id),
        }))
}

/// POST /v1/auth/logout
/// Logout current session
pub async fn logout(
    req: HttpRequest,
    user: AuthenticatedUser,
    auth_service: web::Data<Arc<AuthService>>,
    config: web::Data<crate::config::Config>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    // Get refresh token from cookie
    if let Some(refresh_token) = req.cookie("refresh_token").map(|c| c.value().to_string()) {
        auth_service
            .logout(refresh_token, user.0.sub, ip_address)
            .await?;
    }

    let secure = config.is_production();
    let cookie_domain = config.cookie_domain.as_deref();

    // Clear cookies
    let mut response = HttpResponse::Ok().json(crate::responses::ApiResponse::<()> {
        success: true,
        data: None,
        meta: crate::responses::ResponseMeta::new(request_id),
    });

    for cookie in AuthCookies::clear(secure, cookie_domain) {
        response.add_cookie(&cookie).ok();
    }

    Ok(response)
}

/// POST /v1/auth/logout-all
/// Logout from all sessions
pub async fn logout_all(
    req: HttpRequest,
    user: AuthenticatedUser,
    auth_service: web::Data<Arc<AuthService>>,
    config: web::Data<crate::config::Config>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    auth_service.logout_all(user.0.sub, ip_address).await?;

    let secure = config.is_production();
    let cookie_domain = config.cookie_domain.as_deref();

    let mut response = HttpResponse::Ok().json(crate::responses::ApiResponse::<()> {
        success: true,
        data: None,
        meta: crate::responses::ResponseMeta::new(request_id),
    });

    for cookie in AuthCookies::clear(secure, cookie_domain) {
        response.add_cookie(&cookie).ok();
    }

    Ok(response)
}

/// POST /v1/auth/password-reset
/// Request a password reset
pub async fn request_password_reset(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    email_service: web::Data<Arc<crate::services::EmailService>>,
    body: web::Json<PasswordResetRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    // Validate email format
    crate::validation::validate_email(&body.email)?;

    // Request password reset
    if let Some(token) = auth_service
        .request_password_reset(body.email.clone(), ip_address)
        .await?
    {
        // Send email
        let email = body.email.clone();
        let email_svc = email_service.get_ref().clone();
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_password_reset(&email, &token).await {
                tracing::error!(error = %e, email = %email, "Failed to send password reset email");
            }
        });
    }

    // Always return success (don't reveal if email exists)
    Ok(HttpResponse::Accepted().json(crate::responses::ApiResponse::<()> {
        success: true,
        data: None,
        meta: crate::responses::ResponseMeta::new(request_id),
    }))
}

/// POST /v1/auth/password-reset/confirm
/// Complete password reset with token
pub async fn confirm_password_reset(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<PasswordResetConfirmRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    auth_service
        .complete_password_reset(body.token.clone(), body.new_password.clone(), ip_address)
        .await?;

    Ok(crate::responses::success_no_data(request_id))
}

/// GET /v1/auth/password-reset/verify
/// Verify a password reset token (without using it)
pub async fn verify_password_reset_token(
    req: HttpRequest,
    auth_service: web::Data<Arc<AuthService>>,
    query: web::Query<VerifyMagicLinkRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Just verify the token is valid
    auth_service.verify_reset_token(query.token.clone()).await?;

    Ok(success(serde_json::json!({ "valid": true }), request_id))
}
