//! Authentication service

use chrono::{Duration, Utc};
use ipnetwork::IpNetwork;
use rand::RngCore;
use sqlx::PgPool;
use std::net::IpAddr;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{
    AuditAction, CreateAuditLog, CreateMagicLinkToken, CreatePasswordResetToken,
    CreateRefreshToken, CreateUser, User, UserResponse, UserRole,
};
use crate::repositories::{AuditLogRepository, TokenRepository, UserRepository};
use crate::services::{JwtService, PasswordService};

/// Authentication tokens returned after login
#[derive(Debug, Clone)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Authentication service
pub struct AuthService {
    pool: PgPool,
    jwt: JwtService,
    password: PasswordService,
}

impl AuthService {
    pub fn new(pool: PgPool, jwt: JwtService) -> Self {
        Self {
            pool,
            jwt,
            password: PasswordService::new(),
        }
    }

    /// Register a new user
    pub async fn register(
        &self,
        email: String,
        password: String,
        ip_address: Option<IpAddr>,
    ) -> Result<UserResponse, AppError> {
        // Validate password strength
        self.password.validate_strength(&password)?;
        self.password.validate_not_contains_email(&password, &email)?;

        // Check if email already exists
        if UserRepository::find_by_email(&self.pool, &email)
            .await?
            .is_some()
        {
            return Err(AppError::conflict("Email already registered"));
        }

        // Hash password
        let password_hash = self.password.hash(&password)?;

        // Create user
        let user = UserRepository::create(
            &self.pool,
            CreateUser {
                email: email.clone(),
                password_hash: Some(password_hash),
                role: UserRole::Subscriber,
            },
        )
        .await?;

        // Create audit log
        let ip = ip_address.map(|ip| IpNetwork::from(ip));
        AuditLogRepository::create(
            &self.pool,
            CreateAuditLog::new(AuditAction::UserRegistered)
                .with_actor(user.id, &user.email, &user.role)
                .with_ip(ip)
                .with_resource("user", user.id),
        )
        .await?;

        Ok(UserResponse::from(user))
    }

    /// Login with email and password
    pub async fn login(
        &self,
        email: String,
        password: String,
        device_info: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<(AuthTokens, UserResponse), AppError> {
        // Find user
        let user = UserRepository::find_by_email(&self.pool, &email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Check if user is deleted
        if user.is_deleted() {
            return Err(AppError::InvalidCredentials);
        }

        // Verify password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or(AppError::InvalidCredentials)?;

        if !self.password.verify(&password, password_hash)? {
            return Err(AppError::InvalidCredentials);
        }

        // Create tokens
        let tokens = self.create_tokens(&user, device_info.clone(), ip_address).await?;

        // Update last login
        UserRepository::update_last_login(&self.pool, user.id).await?;

        // Create audit log
        let ip = ip_address.map(|ip| IpNetwork::from(ip));
        AuditLogRepository::create(
            &self.pool,
            CreateAuditLog::new(AuditAction::UserLogin)
                .with_actor(user.id, &user.email, &user.role)
                .with_ip(ip)
                .with_metadata(serde_json::json!({ "device_info": device_info })),
        )
        .await?;

        Ok((tokens, UserResponse::from(user)))
    }

    /// Refresh tokens
    pub async fn refresh_tokens(
        &self,
        refresh_token: String,
        device_info: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<AuthTokens, AppError> {
        // Verify refresh token signature
        let claims = self.jwt.verify_refresh_token(&refresh_token)?;

        // Hash token to find in database
        let token_hash = self.jwt.hash_token(&refresh_token);

        // Find token in database
        let stored_token = TokenRepository::find_refresh_token_by_hash(&self.pool, &token_hash)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Check if token is valid
        if !stored_token.is_valid() {
            return Err(AppError::TokenExpired);
        }

        // Get user
        let user = UserRepository::find_by_id(&self.pool, claims.sub)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Revoke old token
        TokenRepository::revoke_refresh_token(&self.pool, stored_token.id).await?;

        // Create new tokens
        let tokens = self.create_tokens(&user, device_info, ip_address).await?;

        Ok(tokens)
    }

    /// Logout (revoke refresh token)
    pub async fn logout(
        &self,
        refresh_token: String,
        user_id: Uuid,
        ip_address: Option<IpAddr>,
    ) -> Result<(), AppError> {
        // Hash token
        let token_hash = self.jwt.hash_token(&refresh_token);

        // Revoke token
        TokenRepository::revoke_refresh_token_by_hash(&self.pool, &token_hash).await?;

        // Get user for audit log
        if let Some(user) = UserRepository::find_by_id(&self.pool, user_id).await? {
            let ip = ip_address.map(|ip| IpNetwork::from(ip));
            AuditLogRepository::create(
                &self.pool,
                CreateAuditLog::new(AuditAction::UserLogout)
                    .with_actor(user.id, &user.email, &user.role)
                    .with_ip(ip),
            )
            .await?;
        }

        Ok(())
    }

    /// Logout from all sessions
    pub async fn logout_all(&self, user_id: Uuid, ip_address: Option<IpAddr>) -> Result<(), AppError> {
        TokenRepository::revoke_all_user_refresh_tokens(&self.pool, user_id).await?;

        // Get user for audit log
        if let Some(user) = UserRepository::find_by_id(&self.pool, user_id).await? {
            let ip = ip_address.map(|ip| IpNetwork::from(ip));
            AuditLogRepository::create(
                &self.pool,
                CreateAuditLog::new(AuditAction::UserLogout)
                    .with_actor(user.id, &user.email, &user.role)
                    .with_ip(ip)
                    .with_metadata(serde_json::json!({ "all_sessions": true })),
            )
            .await?;
        }

        Ok(())
    }

    /// Request magic link
    pub async fn request_magic_link(
        &self,
        email: String,
        ip_address: Option<IpAddr>,
    ) -> Result<String, AppError> {
        let ip = ip_address.map(|ip| IpNetwork::from(ip));

        // Generate token
        let token = generate_secure_token(32);
        let token_hash = self.jwt.hash_token(&token);
        let expires_at = Utc::now() + Duration::minutes(15);

        // Store token
        TokenRepository::create_magic_link_token(
            &self.pool,
            CreateMagicLinkToken {
                email: email.clone(),
                token_hash,
                expires_at,
                ip_address: ip,
            },
        )
        .await?;

        // Log the request (don't reveal if email exists)
        tracing::info!(email = %email, "Magic link requested");

        Ok(token)
    }

    /// Verify magic link and login
    pub async fn verify_magic_link(
        &self,
        token: String,
        device_info: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<(AuthTokens, UserResponse), AppError> {
        let token_hash = self.jwt.hash_token(&token);

        // Find token
        let magic_token = TokenRepository::find_magic_link_token_by_hash(&self.pool, &token_hash)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        if !magic_token.is_valid() {
            return Err(AppError::TokenExpired);
        }

        // Mark token as used
        TokenRepository::mark_magic_link_token_used(&self.pool, magic_token.id).await?;

        // Find or create user
        let user = match UserRepository::find_by_email(&self.pool, &magic_token.email).await? {
            Some(user) => {
                // Set email as verified
                UserRepository::set_email_verified(&self.pool, user.id).await?;
                UserRepository::find_by_id(&self.pool, user.id).await?.unwrap()
            }
            None => {
                // Create new user (passwordless)
                UserRepository::create(
                    &self.pool,
                    CreateUser {
                        email: magic_token.email.clone(),
                        password_hash: None,
                        role: UserRole::Subscriber,
                    },
                )
                .await?
            }
        };

        // Create tokens
        let tokens = self.create_tokens(&user, device_info, ip_address).await?;

        // Update last login
        UserRepository::update_last_login(&self.pool, user.id).await?;

        // Audit log
        let ip = ip_address.map(|ip| IpNetwork::from(ip));
        AuditLogRepository::create(
            &self.pool,
            CreateAuditLog::new(AuditAction::MagicLinkUsed)
                .with_actor(user.id, &user.email, &user.role)
                .with_ip(ip),
        )
        .await?;

        Ok((tokens, UserResponse::from(user)))
    }

    /// Request password reset
    pub async fn request_password_reset(
        &self,
        email: String,
        ip_address: Option<IpAddr>,
    ) -> Result<Option<String>, AppError> {
        let ip = ip_address.map(|ip| IpNetwork::from(ip));

        // Find user
        let user = match UserRepository::find_by_email(&self.pool, &email).await? {
            Some(user) => user,
            None => return Ok(None), // Don't reveal if email exists
        };

        // Check if user has a password (not magic-link only)
        if user.password_hash.is_none() {
            return Ok(None);
        }

        // Generate token
        let token = generate_secure_token(32);
        let token_hash = self.jwt.hash_token(&token);
        let expires_at = Utc::now() + Duration::hours(1);

        // Store token
        TokenRepository::create_password_reset_token(
            &self.pool,
            CreatePasswordResetToken {
                user_id: user.id,
                token_hash,
                expires_at,
                ip_address: ip,
            },
        )
        .await?;

        // Audit log
        AuditLogRepository::create(
            &self.pool,
            CreateAuditLog::new(AuditAction::PasswordResetRequested)
                .with_actor(user.id, &user.email, &user.role)
                .with_ip(ip),
        )
        .await?;

        Ok(Some(token))
    }

    /// Verify password reset token (check only, don't consume)
    pub async fn verify_reset_token(&self, token: String) -> Result<Uuid, AppError> {
        let token_hash = self.jwt.hash_token(&token);

        let reset_token = TokenRepository::find_password_reset_token_by_hash(&self.pool, &token_hash)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        if !reset_token.is_valid() {
            return Err(AppError::TokenExpired);
        }

        Ok(reset_token.user_id)
    }

    /// Complete password reset
    pub async fn complete_password_reset(
        &self,
        token: String,
        new_password: String,
        ip_address: Option<IpAddr>,
    ) -> Result<(), AppError> {
        // Validate new password
        self.password.validate_strength(&new_password)?;

        let token_hash = self.jwt.hash_token(&token);

        // Find and validate token
        let reset_token = TokenRepository::find_password_reset_token_by_hash(&self.pool, &token_hash)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        if !reset_token.is_valid() {
            return Err(AppError::TokenExpired);
        }

        // Get user
        let user = UserRepository::find_by_id(&self.pool, reset_token.user_id)
            .await?
            .ok_or(AppError::not_found("User"))?;

        // Validate password doesn't contain email
        self.password.validate_not_contains_email(&new_password, &user.email)?;

        // Hash new password
        let password_hash = self.password.hash(&new_password)?;

        // Update password
        UserRepository::update_password(&self.pool, user.id, &password_hash).await?;

        // Mark token as used
        TokenRepository::mark_password_reset_token_used(&self.pool, reset_token.id).await?;

        // Revoke all refresh tokens (logout everywhere)
        TokenRepository::revoke_all_user_refresh_tokens(&self.pool, user.id).await?;

        // Audit log
        let ip = ip_address.map(|ip| IpNetwork::from(ip));
        AuditLogRepository::create(
            &self.pool,
            CreateAuditLog::new(AuditAction::PasswordResetCompleted)
                .with_actor(user.id, &user.email, &user.role)
                .with_ip(ip),
        )
        .await?;

        Ok(())
    }

    /// Change password (for logged-in users)
    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: String,
        new_password: String,
        ip_address: Option<IpAddr>,
    ) -> Result<(), AppError> {
        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::not_found("User"))?;

        // Verify current password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or(AppError::validation("password", "No password set for this account"))?;

        if !self.password.verify(&current_password, password_hash)? {
            return Err(AppError::validation("current_password", "Current password is incorrect"));
        }

        // Validate new password
        self.password.validate_strength(&new_password)?;
        self.password.validate_not_contains_email(&new_password, &user.email)?;

        // Hash and update
        let new_hash = self.password.hash(&new_password)?;
        UserRepository::update_password(&self.pool, user_id, &new_hash).await?;

        // Audit log
        let ip = ip_address.map(|ip| IpNetwork::from(ip));
        AuditLogRepository::create(
            &self.pool,
            CreateAuditLog::new(AuditAction::PasswordChanged)
                .with_actor(user.id, &user.email, &user.role)
                .with_ip(ip),
        )
        .await?;

        Ok(())
    }

    /// Helper to create auth tokens
    async fn create_tokens(
        &self,
        user: &User,
        device_info: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<AuthTokens, AppError> {
        let access_token = self.jwt.create_access_token(user)?;
        let (refresh_token, token_hash) = self.jwt.create_refresh_token(user.id)?;

        let ip = ip_address.map(|ip| IpNetwork::from(ip));
        let expires_at = Utc::now() + Duration::days(30);

        // Store refresh token
        TokenRepository::create_refresh_token(
            &self.pool,
            CreateRefreshToken {
                user_id: user.id,
                token_hash,
                device_info,
                ip_address: ip,
                expires_at,
            },
        )
        .await?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            expires_in: 900, // 15 minutes in seconds
        })
    }
}

/// Generate a cryptographically secure random token
fn generate_secure_token(length: usize) -> String {
    let mut bytes = vec![0u8; length];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes)
}
