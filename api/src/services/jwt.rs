//! JWT token service

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::User;

/// JWT configuration
#[derive(Clone)]
pub struct JwtConfig {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub access_token_expiry: Duration,
    pub refresh_token_expiry: Duration,
    pub issuer: String,
}

impl JwtConfig {
    /// Create config from secret key (for development)
    pub fn from_secret(secret: &str, issuer: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expiry: Duration::minutes(15),
            refresh_token_expiry: Duration::days(30),
            issuer: issuer.to_string(),
        }
    }
}

/// Access token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub membership_status: String,
    pub membership_tier: String,
    pub price_locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_id: Option<String>,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
    pub iss: String,
}

/// Refresh token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: Uuid,
    pub jti: String,
    pub exp: i64,
    pub iat: i64,
}

/// JWT service for token operations
#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// Create access token for a user
    pub fn create_access_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + self.config.access_token_expiry;

        // Get membership tier, defaulting to "personal" if not set
        let membership_tier = user
            .membership_tier
            .clone()
            .unwrap_or_else(|| "personal".to_string());

        let claims = AccessTokenClaims {
            sub: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            membership_status: user.membership_status.clone(),
            membership_tier,
            price_locked: user.price_locked,
            price_id: user.locked_price_id.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            jti: format!("at_{}", Uuid::new_v4().as_simple()),
            iss: self.config.issuer.clone(),
        };

        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.config.encoding_key)
            .map_err(|e| AppError::internal(format!("Failed to create access token: {}", e)))?;

        Ok(token)
    }

    /// Create refresh token
    /// Returns (token, token_hash) - hash is stored in database
    pub fn create_refresh_token(&self, user_id: Uuid) -> Result<(String, String), AppError> {
        let now = Utc::now();
        let exp = now + self.config.refresh_token_expiry;
        let jti = format!("rt_{}", Uuid::new_v4().as_simple());

        let claims = RefreshTokenClaims {
            sub: user_id,
            jti: jti.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.config.encoding_key)
            .map_err(|e| AppError::internal(format!("Failed to create refresh token: {}", e)))?;

        // Hash the token for storage
        let token_hash = self.hash_token(&token);

        Ok((token, token_hash))
    }

    /// Verify access token
    pub fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);

        let token_data = decode::<AccessTokenClaims>(token, &self.config.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
                _ => AppError::InvalidCredentials,
            })?;

        Ok(token_data.claims)
    }

    /// Verify refresh token
    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["sub", "exp"]);
        validation.validate_exp = true;

        let token_data =
            decode::<RefreshTokenClaims>(token, &self.config.decoding_key, &validation).map_err(
                |e| match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
                    _ => AppError::InvalidCredentials,
                },
            )?;

        Ok(token_data.claims)
    }

    /// Decode token without validation (for expired token handling)
    pub fn decode_without_validation(&self, token: &str) -> Result<AccessTokenClaims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();

        let token_data = decode::<AccessTokenClaims>(token, &self.config.decoding_key, &validation)
            .map_err(|_| AppError::InvalidCredentials)?;

        Ok(token_data.claims)
    }

    /// Hash a token for database storage
    pub fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            email_verified: true,
            password_hash: None,
            role: "subscriber".to_string(),
            stripe_customer_id: None,
            membership_status: "active".to_string(),
            membership_tier: Some("personal".to_string()),
            price_locked: false,
            locked_price_id: None,
            locked_price_amount: None,
            grace_period_start: None,
            grace_period_end: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_access_token_creation_and_verification() {
        let config = JwtConfig::from_secret("test-secret-key-12345", "localhost");
        let service = JwtService::new(config);
        let user = create_test_user();

        let token = service.create_access_token(&user).unwrap();
        let claims = service.verify_access_token(&token).unwrap();

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.email, user.email);
        assert_eq!(claims.role, user.role);
    }

    #[test]
    fn test_refresh_token_creation() {
        let config = JwtConfig::from_secret("test-secret-key-12345", "localhost");
        let service = JwtService::new(config);
        let user_id = Uuid::new_v4();

        let (token, hash) = service.create_refresh_token(user_id).unwrap();
        let claims = service.verify_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_token_hashing() {
        let config = JwtConfig::from_secret("test-secret-key-12345", "localhost");
        let service = JwtService::new(config);

        let token = "test-token";
        let hash1 = service.hash_token(token);
        let hash2 = service.hash_token(token);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, token);
    }
}
