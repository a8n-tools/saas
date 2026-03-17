//! TOTP two-factor authentication service

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::TotpRepository;

/// Response from beginning 2FA setup
pub struct TotpSetupInfo {
    pub otpauth_uri: String,
    pub secret: String,
}

/// TOTP service for managing two-factor authentication
pub struct TotpService {
    encryption_key: [u8; 32],
    issuer: String,
    pool: PgPool,
}

impl TotpService {
    pub fn new(encryption_key: [u8; 32], issuer: String, pool: PgPool) -> Self {
        Self {
            encryption_key,
            issuer,
            pool,
        }
    }

    /// Begin 2FA setup: generate a TOTP secret and return the otpauth URI
    pub async fn begin_setup(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> Result<TotpSetupInfo, AppError> {
        let secret = Secret::generate_secret();
        let secret_bytes = secret.to_bytes().map_err(|e| {
            AppError::internal(format!("Failed to generate TOTP secret: {}", e))
        })?;

        let totp = self.build_totp(&secret_bytes, email)?;
        let otpauth_uri = totp.get_url();
        let secret_base32 = data_encoding::BASE32_NOPAD.encode(&secret_bytes);

        // Encrypt and store the secret
        let (encrypted, nonce) = self.encrypt_secret(&secret_bytes)?;
        TotpRepository::upsert_totp(&self.pool, user_id, &encrypted, &nonce).await?;

        Ok(TotpSetupInfo {
            otpauth_uri,
            secret: secret_base32,
        })
    }

    /// Confirm 2FA setup by verifying a TOTP code, then generate recovery codes
    pub async fn confirm_setup(
        &self,
        user_id: Uuid,
        code: &str,
    ) -> Result<Vec<String>, AppError> {
        let totp_record = TotpRepository::find_by_user_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::not_found("TOTP configuration"))?;

        if totp_record.verified {
            return Err(AppError::conflict("2FA is already enabled"));
        }

        // Decrypt secret and verify code
        let secret = self.decrypt_secret(&totp_record.encrypted_secret, &totp_record.nonce)?;
        if !self.check_code(&secret, code)? {
            return Err(AppError::validation("code", "Invalid verification code"));
        }

        // Mark as verified
        TotpRepository::mark_verified(&self.pool, user_id).await?;

        // Generate recovery codes
        let codes = self.generate_and_store_recovery_codes(user_id).await?;
        Ok(codes)
    }

    /// Verify a TOTP code for login
    pub async fn verify_code(&self, user_id: Uuid, code: &str) -> Result<bool, AppError> {
        let totp_record = TotpRepository::find_by_user_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::not_found("TOTP configuration"))?;

        if !totp_record.verified {
            return Ok(false);
        }

        let secret = self.decrypt_secret(&totp_record.encrypted_secret, &totp_record.nonce)?;
        self.check_code(&secret, code)
    }

    /// Verify a recovery code (marks it as used if valid)
    pub async fn verify_recovery_code(
        &self,
        user_id: Uuid,
        code: &str,
    ) -> Result<bool, AppError> {
        let normalized = code.to_uppercase().replace('-', "");
        let hash = Self::hash_code(&normalized);

        match TotpRepository::find_unused_recovery_code(&self.pool, user_id, &hash).await? {
            Some(recovery_code) => {
                TotpRepository::mark_recovery_code_used(&self.pool, recovery_code.id).await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Regenerate recovery codes (requires 2FA to be enabled)
    pub async fn regenerate_recovery_codes(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let totp_record = TotpRepository::find_by_user_id(&self.pool, user_id)
            .await?
            .ok_or(AppError::not_found("TOTP configuration"))?;

        if !totp_record.verified {
            return Err(AppError::validation("2fa", "2FA is not enabled"));
        }

        self.generate_and_store_recovery_codes(user_id).await
    }

    /// Disable 2FA for a user
    pub async fn disable(&self, user_id: Uuid) -> Result<(), AppError> {
        TotpRepository::delete_by_user_id(&self.pool, user_id).await?;
        Ok(())
    }

    /// Check if 2FA is enabled for a user
    pub async fn is_enabled(&self, user_id: Uuid) -> Result<bool, AppError> {
        match TotpRepository::find_by_user_id(&self.pool, user_id).await? {
            Some(record) => Ok(record.verified),
            None => Ok(false),
        }
    }

    /// Get the count of remaining unused recovery codes
    pub async fn recovery_codes_remaining(&self, user_id: Uuid) -> Result<i64, AppError> {
        let count = TotpRepository::count_unused_recovery_codes(&self.pool, user_id).await?;
        Ok(count)
    }

    // --- Internal helpers ---

    fn build_totp(&self, secret: &[u8], account_name: &str) -> Result<TOTP, AppError> {
        TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_vec(),
            Some(self.issuer.clone()),
            account_name.to_string(),
        )
        .map_err(|e| AppError::internal(format!("Failed to create TOTP: {}", e)))
    }

    fn check_code(&self, secret: &[u8], code: &str) -> Result<bool, AppError> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_vec(),
            Some(self.issuer.clone()),
            String::new(),
        )
        .map_err(|e| AppError::internal(format!("Failed to create TOTP for verification: {}", e)))?;

        totp.check_current(code)
            .map_err(|e| AppError::internal(format!("System time error: {}", e)))
    }

    fn encrypt_secret(&self, secret: &[u8]) -> Result<(Vec<u8>, Vec<u8>), AppError> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| AppError::internal(format!("Invalid encryption key: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher
            .encrypt(nonce, secret)
            .map_err(|e| AppError::internal(format!("Encryption failed: {}", e)))?;

        Ok((encrypted, nonce_bytes.to_vec()))
    }

    fn decrypt_secret(&self, encrypted: &[u8], nonce: &[u8]) -> Result<Vec<u8>, AppError> {
        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| AppError::internal(format!("Invalid encryption key: {}", e)))?;

        let nonce = Nonce::from_slice(nonce);

        cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| AppError::internal(format!("Decryption failed: {}", e)))
    }

    async fn generate_and_store_recovery_codes(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let mut codes = Vec::with_capacity(8);
        let mut hashes = Vec::with_capacity(8);

        for _ in 0..8 {
            let code = Self::generate_recovery_code();
            let normalized = code.replace('-', "");
            let hash = Self::hash_code(&normalized);
            codes.push(code);
            hashes.push(hash);
        }

        TotpRepository::insert_recovery_codes(&self.pool, user_id, &hashes).await?;

        Ok(codes)
    }

    fn generate_recovery_code() -> String {
        let mut bytes = [0u8; 4];
        rand::thread_rng().fill_bytes(&mut bytes);
        let hex = hex::encode(bytes).to_uppercase();
        format!("{}-{}", &hex[..4], &hex[4..])
    }

    fn hash_code(code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_encryption_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        key[..16].copy_from_slice(b"test-encrypt-key");
        key[16..].copy_from_slice(b"0123456789abcdef");
        key
    }

    // -- encrypt/decrypt round-trip --

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = test_encryption_key();
        // Cannot use TotpService::new (needs PgPool), test the crypto directly
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();

        let secret = b"my-totp-secret-data";
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher.encrypt(nonce, secret.as_ref()).unwrap();
        let decrypted = cipher.decrypt(nonce, encrypted.as_ref()).unwrap();

        assert_eq!(decrypted, secret);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key1 = test_encryption_key();
        let mut key2 = test_encryption_key();
        key2[0] ^= 0xFF;

        let cipher1 = Aes256Gcm::new_from_slice(&key1).unwrap();
        let cipher2 = Aes256Gcm::new_from_slice(&key2).unwrap();

        let secret = b"secret-data";
        let nonce_bytes = [0u8; 12];
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher1.encrypt(nonce, secret.as_ref()).unwrap();
        assert!(cipher2.decrypt(nonce, encrypted.as_ref()).is_err());
    }

    // -- generate_recovery_code --

    #[test]
    fn recovery_code_format() {
        let code = TotpService::generate_recovery_code();
        // Format: XXXX-XXXX (uppercase hex)
        assert_eq!(code.len(), 9);
        assert_eq!(&code[4..5], "-");
        assert!(code[..4].chars().all(|c| c.is_ascii_hexdigit()));
        assert!(code[5..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn recovery_codes_are_unique() {
        let code1 = TotpService::generate_recovery_code();
        let code2 = TotpService::generate_recovery_code();
        // Extremely unlikely to collide
        assert_ne!(code1, code2);
    }

    // -- hash_code --

    #[test]
    fn hash_code_deterministic() {
        let hash1 = TotpService::hash_code("ABCD1234");
        let hash2 = TotpService::hash_code("ABCD1234");
        assert_eq!(hash1, hash2);
        // SHA256 = 64 hex chars
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn hash_code_different_inputs() {
        let hash1 = TotpService::hash_code("CODE1");
        let hash2 = TotpService::hash_code("CODE2");
        assert_ne!(hash1, hash2);
    }
}
