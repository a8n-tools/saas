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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0xAA; 32]
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = test_key();
        let plaintext = "sk_live_abc123xyz";
        let (ciphertext, nonce) = encrypt_secret(&key, plaintext).unwrap();
        let decrypted = decrypt_secret(&key, &ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_unique_nonces() {
        let key = test_key();
        let (_, nonce1) = encrypt_secret(&key, "secret").unwrap();
        let (_, nonce2) = encrypt_secret(&key, "secret").unwrap();
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key = test_key();
        let wrong_key = [0xBB; 32];
        let (ciphertext, nonce) = encrypt_secret(&key, "secret").unwrap();
        assert!(decrypt_secret(&wrong_key, &ciphertext, &nonce).is_err());
    }

    #[test]
    fn decrypt_tampered_ciphertext_fails() {
        let key = test_key();
        let (mut ciphertext, nonce) = encrypt_secret(&key, "secret").unwrap();
        ciphertext[0] ^= 0xFF;
        assert!(decrypt_secret(&key, &ciphertext, &nonce).is_err());
    }

    #[test]
    fn mask_secret_long_string() {
        assert_eq!(mask_secret("sk_live_abcdefgh1234"), "***1234");
    }

    #[test]
    fn mask_secret_short_strings() {
        assert_eq!(mask_secret(""), "***");
        assert_eq!(mask_secret("abc"), "***");
        assert_eq!(mask_secret("abcd"), "***");
    }

    #[test]
    fn mask_secret_five_chars() {
        assert_eq!(mask_secret("abcde"), "***bcde");
    }

    #[test]
    fn from_db_decrypts_and_masks() {
        let key = test_key();
        let (sk_ct, sk_nonce) = encrypt_secret(&key, "sk_live_test1234").unwrap();
        let (wh_ct, wh_nonce) = encrypt_secret(&key, "whsec_abcdef5678").unwrap();

        let config = StripeConfig {
            id: 1,
            secret_key: Some(sk_ct),
            secret_key_nonce: Some(sk_nonce),
            webhook_secret: Some(wh_ct),
            webhook_secret_nonce: Some(wh_nonce),
            price_id_personal: Some("price_123".to_string()),
            price_id_business: None,
            updated_at: Utc::now(),
            updated_by: None,
        };

        let resp = StripeConfigResponse::from_db(&config, &key).unwrap();
        assert_eq!(resp.secret_key_masked.as_deref(), Some("***1234"));
        assert_eq!(resp.webhook_secret_masked.as_deref(), Some("***5678"));
        assert_eq!(resp.price_id_personal.as_deref(), Some("price_123"));
        assert!(resp.price_id_business.is_none());
        assert!(resp.has_secret_key);
        assert!(resp.has_webhook_secret);
        assert_eq!(resp.source, "database");
    }

    #[test]
    fn from_db_handles_missing_secrets() {
        let key = test_key();
        let config = StripeConfig {
            id: 1,
            secret_key: None,
            secret_key_nonce: None,
            webhook_secret: None,
            webhook_secret_nonce: None,
            price_id_personal: None,
            price_id_business: None,
            updated_at: Utc::now(),
            updated_by: None,
        };

        let resp = StripeConfigResponse::from_db(&config, &key).unwrap();
        assert!(resp.secret_key_masked.is_none());
        assert!(resp.webhook_secret_masked.is_none());
        assert!(!resp.has_secret_key);
        assert!(!resp.has_webhook_secret);
    }
}
