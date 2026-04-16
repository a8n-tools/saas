//! Registry bearer-token handler (`GET /auth/token`).
//!
//! Docker clients call this with basic auth (email:password) after
//! getting a 401+WWW-Authenticate from `/v2/`.

use actix_web::{web, HttpRequest, HttpResponse};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use crate::errors::OciError;
use crate::middleware::extract_client_ip;
use crate::models::{AuditAction, CreateAuditLog, User};
use crate::repositories::{ApplicationRepository, AuditLogRepository, UserRepository};
use crate::services::{OciTokenService, PasswordService};

#[derive(Debug, Deserialize)]
pub struct TokenQuery {
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub access_token: String,
    pub expires_in: u64,
    pub issued_at: String,
}

/// Active-member gate. Mirrors `AccessTokenClaims::has_member_access()` in
/// `api/src/services/jwt.rs` and the OCI bearer extractor check.
fn has_member_access(user: &User) -> bool {
    user.role == "admin"
        || user.lifetime_member
        || user
            .trial_ends_at
            .map_or(false, |t| t > Utc::now())
        || user.membership_status == "active"
        || user.membership_status == "grace_period"
}

/// GET /auth/token
pub async fn issue_token(
    req: HttpRequest,
    query: web::Query<TokenQuery>,
    pool: web::Data<PgPool>,
    token_svc: web::Data<Arc<OciTokenService>>,
) -> Result<HttpResponse, OciError> {
    let ip = extract_client_ip(&req).map(IpNetwork::from);
    let (email, password) = parse_basic_auth(&req).ok_or(OciError::Unauthorized)?;

    let user = UserRepository::find_by_email(pool.get_ref(), &email)
        .await
        .map_err(|_| OciError::Internal)?
        .ok_or(OciError::Unauthorized)?;

    if user.deleted_at.is_some() {
        audit_failed(pool.get_ref(), &email, ip, "inactive_user").await;
        return Err(OciError::Unauthorized);
    }

    // Passwordless accounts (magic-link only) cannot use the registry.
    let Some(password_hash) = user.password_hash.as_ref() else {
        audit_failed(pool.get_ref(), &email, ip, "no_password").await;
        return Err(OciError::Unauthorized);
    };

    let password_service = PasswordService::new();
    let password_ok = password_service
        .verify(&password, password_hash)
        .map_err(|_| OciError::Internal)?;
    if !password_ok {
        audit_failed(pool.get_ref(), &email, ip, "bad_password").await;
        return Err(OciError::Unauthorized);
    }

    if !has_member_access(&user) {
        audit_failed(pool.get_ref(), &email, ip, "no_active_membership").await;
        return Err(OciError::Unauthorized);
    }

    // Scope validation: if provided, the target app must exist + be pullable.
    let mut scope_str = String::new();
    if let Some(raw_scope) = &query.scope {
        let slug = parse_repository_pull_scope(raw_scope).ok_or(OciError::Denied)?;
        let app = ApplicationRepository::find_active_by_slug(pool.get_ref(), &slug)
            .await
            .map_err(|_| OciError::Internal)?
            .ok_or(OciError::NameUnknown)?;
        if !app.is_pullable() {
            return Err(OciError::NameUnknown);
        }
        scope_str = format!("repository:{slug}:pull");
    }

    let token = token_svc.issue(user.id, &scope_str)?;
    let now = Utc::now();

    let log = CreateAuditLog::new(AuditAction::OciLoginSucceeded)
        .with_actor(user.id, &user.email, &user.role)
        .with_ip(ip)
        .with_metadata(serde_json::json!({ "scope": scope_str }));
    let _ = AuditLogRepository::create(pool.get_ref(), log).await;

    Ok(HttpResponse::Ok().json(TokenResponse {
        token: token.clone(),
        access_token: token,
        expires_in: token_svc.ttl_secs(),
        issued_at: now.to_rfc3339(),
    }))
}

fn parse_basic_auth(req: &HttpRequest) -> Option<(String, String)> {
    let header = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let b64 = header.strip_prefix("Basic ")?;
    let decoded = STANDARD.decode(b64).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (email, password) = decoded.split_once(':')?;
    Some((email.to_string(), password.to_string()))
}

fn parse_repository_pull_scope(scope: &str) -> Option<String> {
    // Docker sends scopes like "repository:my-app:pull" (possibly comma-separated).
    // We accept only single-repo pull scopes.
    let (kind, rest) = scope.split_once(':')?;
    if kind != "repository" {
        return None;
    }
    let (slug, action) = rest.rsplit_once(':')?;
    if action != "pull" {
        return None;
    }
    Some(slug.to_string())
}

async fn audit_failed(
    pool: &PgPool,
    email: &str,
    ip: Option<IpNetwork>,
    reason: &str,
) {
    let log = CreateAuditLog::new(AuditAction::OciLoginFailed)
        .with_ip(ip)
        .with_metadata(serde_json::json!({ "email": email, "reason": reason }));
    let _ = AuditLogRepository::create(pool, log).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scope_accepts_repository_pull() {
        assert_eq!(
            parse_repository_pull_scope("repository:my-app:pull"),
            Some("my-app".into())
        );
        assert_eq!(
            parse_repository_pull_scope("repository:complex/slug:pull"),
            Some("complex/slug".into())
        );
        assert!(parse_repository_pull_scope("repository:my-app:push").is_none());
        assert!(parse_repository_pull_scope("registry:catalog:*").is_none());
        assert!(parse_repository_pull_scope("repository:my-app").is_none());
    }

    #[test]
    fn parse_basic_auth_decodes_header() {
        let req = actix_web::test::TestRequest::default()
            .insert_header((
                "Authorization",
                format!("Basic {}", STANDARD.encode("me@example.com:hunter2")),
            ))
            .to_http_request();
        assert_eq!(
            parse_basic_auth(&req),
            Some(("me@example.com".into(), "hunter2".into()))
        );
    }
}
