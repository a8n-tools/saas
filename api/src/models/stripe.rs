use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Debug, Clone, FromRow)]
pub struct StripeConfig {
    pub id: i32,
    pub secret_key: Option<Vec<u8>>,
    pub secret_key_nonce: Option<Vec<u8>>,
    pub webhook_secret: Option<Vec<u8>>,
    pub webhook_secret_nonce: Option<Vec<u8>>,
    pub price_id_personal: Option<String>,
    pub price_id_business: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// Encrypt plaintext with AES-256-GCM. Returns (ciphertext, nonce).
pub fn encrypt_secret(key: &[u8; 32], plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::internal(format!("Invalid encryption key: {}", e)))?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let encrypted = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::internal(format!("Encryption failed: {}", e)))?;
    Ok((encrypted, nonce_bytes.to_vec()))
}

/// Decrypt AES-256-GCM ciphertext. Returns plaintext string.
pub fn decrypt_secret(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::internal(format!("Invalid encryption key: {}", e)))?;
    let nonce = Nonce::from_slice(nonce);
    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::internal(format!("Decryption failed: {}", e)))?;
    String::from_utf8(decrypted)
        .map_err(|e| AppError::internal(format!("Invalid UTF-8 in decrypted secret: {}", e)))
}

/// Returns `***<last 4 chars>` so admins can identify which key is stored.
pub fn mask_secret(s: &str) -> String {
    if s.len() <= 4 {
        return "***".to_string();
    }
    format!("***{}", &s[s.len() - 4..])
}

#[derive(Debug, Serialize)]
pub struct StripeConfigResponse {
    pub secret_key_masked: Option<String>,
    pub webhook_secret_masked: Option<String>,
    pub price_id_personal: Option<String>,
    pub price_id_business: Option<String>,
    pub has_secret_key: bool,
    pub has_webhook_secret: bool,
    pub updated_at: Option<DateTime<Utc>>,
    /// "database" or "environment" — indicates where the config came from
    pub source: String,
}

impl StripeConfigResponse {
    pub fn from_db(config: &StripeConfig, key: &[u8; 32]) -> Result<Self, AppError> {
        let secret_key_plain = match (&config.secret_key, &config.secret_key_nonce) {
            (Some(ct), Some(nonce)) => Some(decrypt_secret(key, ct, nonce)?),
            _ => None,
        };
        let webhook_secret_plain = match (&config.webhook_secret, &config.webhook_secret_nonce) {
            (Some(ct), Some(nonce)) => Some(decrypt_secret(key, ct, nonce)?),
            _ => None,
        };

        Ok(Self {
            secret_key_masked: secret_key_plain.as_deref().map(mask_secret),
            webhook_secret_masked: webhook_secret_plain.as_deref().map(mask_secret),
            price_id_personal: config.price_id_personal.clone(),
            price_id_business: config.price_id_business.clone(),
            has_secret_key: config.secret_key.is_some(),
            has_webhook_secret: config.webhook_secret.is_some(),
            updated_at: Some(config.updated_at),
            source: "database".to_string(),
        })
    }

    /// Reads env vars and returns a response showing what's currently configured there.
    /// Used as a fallback when no DB config has been saved yet.
    pub fn from_env() -> Self {
        use std::env;
        let secret_key = env::var("STRIPE_SECRET_KEY").ok().filter(|s| !s.is_empty());
        let webhook_secret = env::var("STRIPE_WEBHOOK_SECRET").ok().filter(|s| !s.is_empty());
        let price_id_personal = env::var("STRIPE_PRICE_ID").ok().filter(|s| !s.is_empty());
        let price_id_business = env::var("STRIPE_BUSINESS_PRICE_ID").ok().filter(|s| !s.is_empty());

        Self {
            secret_key_masked: secret_key.as_deref().map(mask_secret),
            webhook_secret_masked: webhook_secret.as_deref().map(mask_secret),
            price_id_personal,
            price_id_business,
            has_secret_key: secret_key.is_some(),
            has_webhook_secret: webhook_secret.is_some(),
            updated_at: None,
            source: "environment".to_string(),
        }
    }
}
