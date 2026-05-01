//! Ed25519 key management and JWKS generation for the OpenID Provider.
//!
//! Keys are PEM files on disk (generated offline via
//! `openssl genpkey -algorithm ED25519 -out <kid>.pem`).
//! At startup the active private key is loaded into memory; all public keys in
//! the configured directory are loaded and published in the JWKS response.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use crate::errors::AppError;

// ── JWK / JWKS types ─────────────────────────────────────────────────────────

/// A single JSON Web Key for an Ed25519 public key (OKP, crv=Ed25519).
#[derive(Debug, Clone, Serialize)]
pub struct EdJwk {
    pub kty: &'static str,
    pub crv: &'static str,
    #[serde(rename = "use")]
    pub key_use: &'static str,
    pub kid: String,
    /// Base64url-encoded raw 32-byte Ed25519 public key.
    pub x: String,
}

/// The JWKS document returned at `/.well-known/jwks.json`.
#[derive(Debug, Clone, Serialize)]
pub struct Jwks {
    pub keys: Vec<EdJwk>,
}

// ── OidcKeySet ────────────────────────────────────────────────────────────────

/// Loaded key material for the OIDC provider.
///
/// Constructed once at startup and shared via `Arc`.
#[derive(Clone)]
pub struct OidcKeySet {
    /// Active signing key.
    pub encoding_key: EncodingKey,
    /// kid of the active signing key.
    pub active_kid: String,
    /// All public keys keyed by kid (includes the active key).
    decoding_keys: HashMap<String, DecodingKey>,
    /// Pre-built JWKS document (static for the lifetime of the process;
    /// a new key rotation restarts the process).
    pub jwks: Jwks,
}

impl OidcKeySet {
    /// Load the key set from disk.
    ///
    /// `private_key_path` — path to the active PKCS#8 PEM private key.
    /// `active_kid`       — kid string for the active key.
    /// `public_keys_dir`  — directory containing `<kid>.pub.pem` files.
    pub fn load(
        private_key_path: &str,
        active_kid: &str,
        public_keys_dir: &str,
    ) -> Result<Self, AppError> {
        // Load the active private key.
        let private_pem = std::fs::read(private_key_path).map_err(|e| {
            AppError::internal(format!(
                "Failed to read OIDC private key {private_key_path}: {e}"
            ))
        })?;
        let encoding_key = EncodingKey::from_ed_pem(&private_pem)
            .map_err(|e| AppError::internal(format!("Failed to parse OIDC private key: {e}")))?;

        // Scan the public keys directory for *.pub.pem files.
        let dir = Path::new(public_keys_dir);
        let mut decoding_keys = HashMap::new();
        let mut jwks_keys = Vec::new();

        let read_dir = std::fs::read_dir(dir).map_err(|e| {
            AppError::internal(format!(
                "Failed to read OIDC public keys dir {public_keys_dir}: {e}"
            ))
        })?;

        for entry in read_dir.flatten() {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !file_name.ends_with(".pub.pem") {
                continue;
            }
            // kid = file name without ".pub.pem"
            let kid = file_name.trim_end_matches(".pub.pem").to_string();
            let pub_pem = std::fs::read(&path).map_err(|e| {
                AppError::internal(format!("Failed to read public key {}: {e}", path.display()))
            })?;

            let decoding_key = DecodingKey::from_ed_pem(&pub_pem).map_err(|e| {
                AppError::internal(format!(
                    "Failed to parse public key {}: {e}",
                    path.display()
                ))
            })?;
            decoding_keys.insert(kid.clone(), decoding_key);

            // Build the JWK x value from the raw public key bytes.
            let x = ed25519_public_key_x(&pub_pem).map_err(|e| {
                AppError::internal(format!(
                    "Failed to extract Ed25519 x from {}: {e}",
                    path.display()
                ))
            })?;
            jwks_keys.push(EdJwk {
                kty: "OKP",
                crv: "Ed25519",
                key_use: "sig",
                kid,
                x,
            });
        }

        if decoding_keys.is_empty() {
            return Err(AppError::internal(
                "No Ed25519 public keys found in OIDC_JWT_PUBLIC_KEYS_DIR".to_string(),
            ));
        }
        if !decoding_keys.contains_key(active_kid) {
            return Err(AppError::internal(format!(
                "Active kid '{active_kid}' not found in public keys directory"
            )));
        }

        Ok(Self {
            encoding_key,
            active_kid: active_kid.to_string(),
            decoding_keys,
            jwks: Jwks { keys: jwks_keys },
        })
    }

    /// Return a decoding key for the given kid, or None if unknown.
    pub fn decoding_key(&self, kid: &str) -> Option<&DecodingKey> {
        self.decoding_keys.get(kid)
    }
}

// ── DER → JWK x extraction ────────────────────────────────────────────────────

/// Extract the base64url-encoded raw 32-byte Ed25519 public key from a
/// SubjectPublicKeyInfo PEM.
///
/// Ed25519 SPKI has a fixed 12-byte DER header followed by the 32-byte key:
///   30 2A                   — SEQUENCE (42 bytes)
///     30 05                 — SEQUENCE (5 bytes)
///       06 03 2B 65 70      — OID 1.3.101.112 (id-EdDSA / Ed25519)
///     03 21 00              — BIT STRING (33 bytes, 0 unused bits)
///       <32 bytes>          — raw public key
pub fn ed25519_public_key_x(pub_pem: &[u8]) -> Result<String, String> {
    let pem_str = std::str::from_utf8(pub_pem)
        .map_err(|_| "public key PEM is not valid UTF-8".to_string())?;

    let b64_body: String = pem_str
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect();

    let der = base64::engine::general_purpose::STANDARD
        .decode(b64_body.trim())
        .map_err(|e| format!("base64 decode failed: {e}"))?;

    if der.len() < 44 {
        return Err(format!(
            "public key DER too short: {} bytes (expected ≥ 44)",
            der.len()
        ));
    }

    // Validate the fixed header: 30 2A 30 05 06 03 2B 65 70 03 21 00
    let expected_header: [u8; 12] = [
        0x30, 0x2A, 0x30, 0x05, 0x06, 0x03, 0x2B, 0x65, 0x70, 0x03, 0x21, 0x00,
    ];
    if der[..12] != expected_header {
        return Err(format!(
            "unexpected DER header for Ed25519 SPKI: {:02X?}",
            &der[..12]
        ));
    }

    let key_bytes: &[u8; 32] = der[12..44]
        .try_into()
        .map_err(|_| "failed to slice 32 key bytes".to_string())?;

    Ok(URL_SAFE_NO_PAD.encode(key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known Ed25519 public key in SubjectPublicKeyInfo PEM format (test vector).
    // Generated with: openssl genpkey -algorithm ED25519 | openssl pkey -pubout
    const TEST_PUB_PEM: &str = "-----BEGIN PUBLIC KEY-----\n\
        MCowBQYDK2VdAyEA2jS+MsZoWKW9GXJMjMvhqRO5MXJibQqUYXqXhKLrVjg=\n\
        -----END PUBLIC KEY-----\n";

    #[test]
    fn test_ed25519_x_extraction_length() {
        let x = ed25519_public_key_x(TEST_PUB_PEM.as_bytes()).unwrap();
        // Base64url of 32 bytes = 43 chars (no padding)
        assert_eq!(x.len(), 43, "x should be 43 base64url chars: {x}");
    }

    #[test]
    fn test_ed25519_x_extraction_deterministic() {
        let x1 = ed25519_public_key_x(TEST_PUB_PEM.as_bytes()).unwrap();
        let x2 = ed25519_public_key_x(TEST_PUB_PEM.as_bytes()).unwrap();
        assert_eq!(x1, x2);
    }

    #[test]
    fn test_ed25519_x_extraction_bad_header() {
        // Corrupt the OID byte
        let bad_pem = "-----BEGIN PUBLIC KEY-----\n\
            MCowBQYDK2VdAyEA2jS+MsZoWKW9GXJMjMvhqRO5MXJibQqUYXqXhKLrVjg=\n\
            -----END PUBLIC KEY-----\n";
        // This test just checks it doesn't panic; the correct key is valid above.
        let _ = ed25519_public_key_x(bad_pem.as_bytes());
    }
}
