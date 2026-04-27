//! Registry bearer-token issuance and verification.
//!
//! Tokens reuse the platform JWT keypair with `aud="registry"`. Scope
//! claim carries `repository:<slug>:pull` or is empty for a bare
//! service probe.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::OciError;
use crate::services::JwtConfig;

pub const REGISTRY_AUDIENCE: &str = "registry";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryTokenClaims {
    pub sub: Uuid,
    pub aud: String,
    #[serde(default)]
    pub scope: String,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
}

#[derive(Clone)]
pub struct OciTokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    ttl: Duration,
}

impl OciTokenService {
    pub fn new(jwt_config: &JwtConfig, ttl_secs: u64) -> Self {
        Self {
            encoding_key: jwt_config.encoding_key.clone(),
            decoding_key: jwt_config.decoding_key.clone(),
            issuer: jwt_config.issuer.clone(),
            ttl: Duration::seconds(ttl_secs as i64),
        }
    }

    pub fn issue(&self, user_id: Uuid, scope: &str) -> Result<String, OciError> {
        let now = Utc::now();
        let claims = RegistryTokenClaims {
            sub: user_id,
            aud: REGISTRY_AUDIENCE.into(),
            scope: scope.to_string(),
            iat: now.timestamp(),
            exp: (now + self.ttl).timestamp(),
            iss: self.issuer.clone(),
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(|_| OciError::Internal)
    }

    pub fn verify(&self, token: &str) -> Result<RegistryTokenClaims, OciError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[REGISTRY_AUDIENCE]);
        validation.set_issuer(&[&self.issuer]);
        validation.leeway = 0;
        let data = decode::<RegistryTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|_| OciError::Unauthorized)?;
        Ok(data.claims)
    }

    pub fn ttl_secs(&self) -> u64 {
        self.ttl.num_seconds() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svc() -> OciTokenService {
        let cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        OciTokenService::new(&cfg, 900)
    }

    #[test]
    fn roundtrip_issue_and_verify() {
        let svc = svc();
        let user = Uuid::new_v4();
        let tok = svc.issue(user, "repository:my-app:pull").unwrap();
        let claims = svc.verify(&tok).unwrap();
        assert_eq!(claims.sub, user);
        assert_eq!(claims.aud, "registry");
        assert_eq!(claims.scope, "repository:my-app:pull");
    }

    #[test]
    fn rejects_token_with_wrong_audience() {
        let cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let now = Utc::now();
        let bad_claims = RegistryTokenClaims {
            sub: Uuid::new_v4(),
            aud: "api".into(),
            scope: "".into(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(900)).timestamp(),
            iss: "a8n".into(),
        };
        let bad = encode(&Header::new(Algorithm::HS256), &bad_claims, &cfg.encoding_key).unwrap();

        let svc = svc();
        assert!(matches!(svc.verify(&bad), Err(OciError::Unauthorized)));
    }

    #[test]
    fn rejects_expired_token() {
        let cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let past = Utc::now() - Duration::seconds(10);
        let claims = RegistryTokenClaims {
            sub: Uuid::new_v4(),
            aud: "registry".into(),
            scope: "".into(),
            iat: past.timestamp() - 5,
            exp: past.timestamp(),
            iss: "a8n".into(),
        };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &cfg.encoding_key).unwrap();

        let svc = svc();
        assert!(matches!(svc.verify(&token), Err(OciError::Unauthorized)));
    }
}
