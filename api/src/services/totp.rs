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
