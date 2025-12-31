//! Password hashing service

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};

use crate::errors::AppError;
use crate::validation::validate_password_strength;

/// Password service for hashing and verification
pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl PasswordService {
    /// Create a new password service with recommended Argon2id parameters
    pub fn new() -> Self {
        // Recommended parameters for Argon2id
        // Memory: 64 MiB, Iterations: 3, Parallelism: 4
        let params = Params::new(
            64 * 1024, // 64 MiB memory
            3,         // 3 iterations
            4,         // 4 parallelism
            None,      // default output length
        )
        .expect("Invalid Argon2 parameters");

        Self {
            argon2: Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params),
        }
    }

    /// Hash a password
    pub fn hash(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);

        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::internal(format!("Password hashing failed: {}", e)))?;

        Ok(hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::internal(format!("Invalid password hash format: {}", e)))?;

        Ok(self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Validate password strength
    pub fn validate_strength(&self, password: &str) -> Result<(), AppError> {
        validate_password_strength(password).map_err(|e| {
            let message = e.message.map(|m| m.to_string()).unwrap_or_else(|| {
                "Password does not meet strength requirements".to_string()
            });
            AppError::validation("password", message)
        })
    }

    /// Validate password doesn't contain the email
    pub fn validate_not_contains_email(&self, password: &str, email: &str) -> Result<(), AppError> {
        let email_parts: Vec<&str> = email.split('@').collect();
        if let Some(username) = email_parts.first() {
            if username.len() >= 4 && password.to_lowercase().contains(&username.to_lowercase()) {
                return Err(AppError::validation(
                    "password",
                    "Password cannot contain your email address",
                ));
            }
        }
        Ok(())
    }
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let service = PasswordService::new();
        let password = "SecurePassword123!";

        let hash = service.hash(password).unwrap();
        assert!(service.verify(password, &hash).unwrap());
        assert!(!service.verify("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_hash_uniqueness() {
        let service = PasswordService::new();
        let password = "SecurePassword123!";

        let hash1 = service.hash(password).unwrap();
        let hash2 = service.hash(password).unwrap();

        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(service.verify(password, &hash1).unwrap());
        assert!(service.verify(password, &hash2).unwrap());
    }

    #[test]
    fn test_validate_strength() {
        let service = PasswordService::new();

        assert!(service.validate_strength("SecurePass123!").is_ok());
        assert!(service.validate_strength("weak").is_err());
    }

    #[test]
    fn test_validate_not_contains_email() {
        let service = PasswordService::new();

        assert!(service
            .validate_not_contains_email("SecurePass123!", "user@example.com")
            .is_ok());
        assert!(service
            .validate_not_contains_email("userPassword123!", "user@example.com")
            .is_err());
    }
}
