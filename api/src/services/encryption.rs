//! Shared encryption key management with key rotation support.
//!
//! Provides [`EncryptionKeySet`] which centralises AES-256-GCM encrypt/decrypt
//! with automatic fallback to a previous key during rotation windows.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use crate::errors::AppError;

/// A current + optional previous encryption key pair for zero-downtime rotation.
#[derive(Clone)]
pub struct EncryptionKeySet {
    pub current: [u8; 32],
    pub current_version: i16,
    pub previous: Option<[u8; 32]>,
}

impl EncryptionKeySet {
    /// Encrypt plaintext with the current key.
    /// Returns `(ciphertext, nonce, key_version)`.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>, i16), AppError> {
        let cipher = Aes256Gcm::new_from_slice(&self.current)
            .map_err(|e| AppError::internal(format!("Invalid encryption key: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| AppError::internal(format!("Encryption failed: {e}")))?;

        Ok((ciphertext, nonce_bytes.to_vec(), self.current_version))
    }

    /// Decrypt ciphertext, using the key that matches `key_version`.
    ///
    /// If `key_version` matches `current_version`, tries the current key first.
    /// On mismatch (or failure with a previous key available), falls back to the
    /// other key. This allows seamless decryption during a rotation window.
    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &[u8],
        key_version: i16,
    ) -> Result<Vec<u8>, AppError> {
        let nonce = Nonce::from_slice(nonce);

        if key_version == self.current_version {
            // Expected path: version matches current key.
            let cipher = Aes256Gcm::new_from_slice(&self.current)
                .map_err(|e| AppError::internal(format!("Invalid encryption key: {e}")))?;
            match cipher.decrypt(nonce, ciphertext) {
                Ok(plaintext) => return Ok(plaintext),
                Err(_) if self.previous.is_some() => {
                    // Current key failed despite matching version — try previous as fallback.
                }
                Err(e) => {
                    return Err(AppError::internal(format!("Decryption failed: {e}")));
                }
            }
        }

        // Try previous key (version mismatch or current-key failure).
        if let Some(prev) = &self.previous {
            let cipher = Aes256Gcm::new_from_slice(prev)
                .map_err(|e| AppError::internal(format!("Invalid previous encryption key: {e}")))?;
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| AppError::internal(format!("Decryption failed with both keys: {e}")))
        } else {
            // No previous key and current key didn't match version.
            let cipher = Aes256Gcm::new_from_slice(&self.current)
                .map_err(|e| AppError::internal(format!("Invalid encryption key: {e}")))?;
            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| AppError::internal(format!("Decryption failed: {e}")))
        }
    }

    /// Returns `true` if a record encrypted with `key_version` needs re-encryption.
    pub fn needs_reencrypt(&self, key_version: i16) -> bool {
        key_version != self.current_version
    }
}

/// Decrypt ciphertext with a single key (no fallback). Used by key health checks
/// and other contexts where we want to test a specific key.
pub fn decrypt_with_key(
    key: &[u8; 32],
    ciphertext: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, AppError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::internal(format!("Invalid encryption key: {e}")))?;
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::internal(format!("Decryption failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_a() -> [u8; 32] {
        [0xAA; 32]
    }

    fn key_b() -> [u8; 32] {
        [0xBB; 32]
    }

    fn make_key_set(current: [u8; 32], version: i16, previous: Option<[u8; 32]>) -> EncryptionKeySet {
        EncryptionKeySet {
            current,
            current_version: version,
            previous,
        }
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let ks = make_key_set(key_a(), 1, None);
        let plaintext = b"hello world";

        let (ct, nonce, version) = ks.encrypt(plaintext).unwrap();
        assert_eq!(version, 1);

        let decrypted = ks.decrypt(&ct, &nonce, 1).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_previous_key_on_version_mismatch() {
        // Encrypt with key_a (version 1)
        let ks_v1 = make_key_set(key_a(), 1, None);
        let (ct, nonce, _) = ks_v1.encrypt(b"secret").unwrap();

        // Now rotate: key_b is current (version 2), key_a is previous
        let ks_v2 = make_key_set(key_b(), 2, Some(key_a()));
        let decrypted = ks_v2.decrypt(&ct, &nonce, 1).unwrap();
        assert_eq!(decrypted, b"secret");
    }

    #[test]
    fn decrypt_current_key_version_match() {
        let ks = make_key_set(key_b(), 2, Some(key_a()));
        let (ct, nonce, version) = ks.encrypt(b"new secret").unwrap();
        assert_eq!(version, 2);

        let decrypted = ks.decrypt(&ct, &nonce, 2).unwrap();
        assert_eq!(decrypted, b"new secret");
    }

    #[test]
    fn decrypt_fails_with_wrong_keys() {
        let ks_v1 = make_key_set(key_a(), 1, None);
        let (ct, nonce, _) = ks_v1.encrypt(b"secret").unwrap();

        // key_b as current, no previous — should fail
        let ks_wrong = make_key_set(key_b(), 2, None);
        assert!(ks_wrong.decrypt(&ct, &nonce, 1).is_err());
    }

    #[test]
    fn needs_reencrypt_detects_old_version() {
        let ks = make_key_set(key_b(), 2, Some(key_a()));
        assert!(ks.needs_reencrypt(1));
        assert!(!ks.needs_reencrypt(2));
    }

    #[test]
    fn encrypt_produces_unique_nonces() {
        let ks = make_key_set(key_a(), 1, None);
        let (_, nonce1, _) = ks.encrypt(b"a").unwrap();
        let (_, nonce2, _) = ks.encrypt(b"a").unwrap();
        assert_ne!(nonce1, nonce2);
    }
}
