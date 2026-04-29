//! /v2/* handlers for the OCI registry server.

use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use ipnetwork::IpNetwork;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::sync::Arc;
use tokio_util::codec::{BytesCodec, FramedRead};
use uuid::Uuid;

use crate::errors::{AppError, OciError};
use crate::middleware::{extract_client_ip, OciBearerUser};
use crate::models::oci::CachedManifest;
use crate::models::{AuditAction, CreateAuditLog};
use crate::repositories::{ApplicationRepository, AuditLogRepository};
use crate::services::forgejo_registry::{ForgejoRegistryClient, RegistryError};
use crate::services::{BlobCache, ManifestCache, OciLimitDenial, OciLimiter};

const DEFAULT_ACCEPT: &str = "application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json";

/// GET /v2/  — version probe. Requires auth but no scope.
pub async fn version_probe(user: Option<OciBearerUser>) -> Result<HttpResponse, OciError> {
    match user {
        Some(_) => Ok(HttpResponse::Ok()
            .append_header(("Docker-Distribution-API-Version", "registry/2.0"))
            .finish()),
        None => Err(OciError::Unauthorized),
    }
}

/// GET/HEAD /v2/{slug}/manifests/{reference}
pub async fn get_manifest(
    req: HttpRequest,
    user: OciBearerUser,
    path: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    client: web::Data<Option<Arc<ForgejoRegistryClient>>>,
    manifest_cache: web::Data<Option<Arc<ManifestCache>>>,
    limiter: web::Data<Arc<OciLimiter>>,
) -> Result<HttpResponse, OciError> {
    let (slug, reference) = path.into_inner();
    if user.assert_scope(&slug).is_err() {
        audit_denied_scope(pool.get_ref(), &req, &user, &slug).await;
        return Err(OciError::Denied);
    }

    let client = client
        .as_ref()
        .as_ref()
        .ok_or(OciError::NameUnknown)?
        .clone();
    let cache = manifest_cache
        .as_ref()
        .as_ref()
        .ok_or(OciError::Internal)?
        .clone();

    let app = ApplicationRepository::find_active_by_slug(pool.get_ref(), &slug)
        .await
        .map_err(|_| OciError::Internal)?
        .ok_or(OciError::NameUnknown)?;
    if !app.is_pullable() {
        return Err(OciError::NameUnknown);
    }
    let pinned = app
        .pinned_image_tag
        .clone()
        .ok_or(OciError::ManifestUnknown)?;

    // Reference must be the pinned tag or a sha256 digest. Child manifests
    // (multi-arch) are fetched via digest references and hash-verified by
    // upstream; we allow them through.
    let is_digest = reference.starts_with("sha256:");
    if !is_digest && reference != pinned {
        return Err(OciError::ManifestUnknown);
    }

    let guard = match limiter
        .acquire(pool.get_ref(), user.claims.sub)
        .await
        .map_err(|_| OciError::Internal)?
    {
        Ok(g) => g,
        Err(OciLimitDenial::Concurrency) => {
            audit_denied(pool.get_ref(), &req, &user, &app.id, "concurrency", None).await;
            return Err(OciError::TooManyRequests {
                retry_after_secs: None,
            });
        }
        Err(OciLimitDenial::DailyCap { reset_in_secs }) => {
            let secs_u64 = reset_in_secs.max(0) as u64;
            audit_denied(
                pool.get_ref(),
                &req,
                &user,
                &app.id,
                "daily_cap",
                Some(secs_u64),
            )
            .await;
            return Err(OciError::TooManyRequests {
                retry_after_secs: Some(secs_u64),
            });
        }
    };

    audit_requested(pool.get_ref(), &req, &user, &app.id, &reference).await;

    let accept = req
        .headers()
        .get(actix_web::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or(DEFAULT_ACCEPT)
        .to_string();

    let manifest: Arc<CachedManifest> = if let Some(hit) = cache.get(app.id, &reference).await {
        hit
    } else {
        let owner = app
            .oci_image_owner
            .as_deref()
            .ok_or(OciError::NameUnknown)?;
        let name = app.oci_image_name.as_deref().ok_or(OciError::NameUnknown)?;
        let mr = match client.get_manifest(owner, name, &reference, &accept).await {
            Ok(mr) => mr,
            Err(e) => {
                let mapped = map_reg_err(&e);
                if matches!(mapped, OciError::Upstream) {
                    audit_failed_upstream(
                        pool.get_ref(),
                        &req,
                        &user,
                        &app.id,
                        "manifest",
                        &reference,
                        &format!("{e:?}"),
                    )
                    .await;
                }
                return Err(mapped);
            }
        };
        let digest = if mr.digest.is_empty() {
            format!("sha256:{}", hex::encode(Sha256::digest(&mr.bytes)))
        } else {
            mr.digest
        };
        cache
            .insert(
                app.id,
                &reference,
                CachedManifest {
                    bytes: mr.bytes,
                    media_type: mr.media_type,
                    digest,
                },
            )
            .await
    };

    audit_completed(
        pool.get_ref(),
        &req,
        &user,
        &app.id,
        &reference,
        &manifest.digest,
    )
    .await;
    drop(guard);

    let is_head = req.method() == actix_web::http::Method::HEAD;
    let mut resp = HttpResponse::Ok();
    resp.insert_header(("Content-Type", manifest.media_type.clone()));
    resp.insert_header(("Docker-Content-Digest", manifest.digest.clone()));
    resp.insert_header(("Content-Length", manifest.bytes.len().to_string()));
    if is_head {
        Ok(resp.finish())
    } else {
        Ok(resp.body(manifest.bytes.clone()))
    }
}

/// GET/HEAD /v2/{slug}/blobs/{digest}
pub async fn get_blob(
    req: HttpRequest,
    user: OciBearerUser,
    path: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    blob_cache: web::Data<Option<Arc<BlobCache>>>,
) -> Result<HttpResponse, OciError> {
    let (slug, digest) = path.into_inner();
    if user.assert_scope(&slug).is_err() {
        audit_denied_scope(pool.get_ref(), &req, &user, &slug).await;
        return Err(OciError::Denied);
    }
    let blob_cache = blob_cache
        .as_ref()
        .as_ref()
        .ok_or(OciError::Internal)?
        .clone();

    let app = ApplicationRepository::find_active_by_slug(pool.get_ref(), &slug)
        .await
        .map_err(|_| OciError::Internal)?
        .ok_or(OciError::NameUnknown)?;
    if !app.is_pullable() {
        return Err(OciError::NameUnknown);
    }
    let owner = app
        .oci_image_owner
        .as_deref()
        .ok_or(OciError::NameUnknown)?;
    let name = app.oci_image_name.as_deref().ok_or(OciError::NameUnknown)?;

    let handle = match blob_cache.get_or_fetch(owner, name, &digest).await {
        Ok(h) => h,
        Err(e) => {
            let mapped = match &e {
                AppError::NotFound { .. } => OciError::BlobUnknown,
                AppError::ValidationError { .. } => OciError::BlobUnknown,
                _ => OciError::Upstream,
            };
            if matches!(mapped, OciError::Upstream) {
                audit_failed_upstream(
                    pool.get_ref(),
                    &req,
                    &user,
                    &app.id,
                    "blob",
                    &digest,
                    &format!("{e:?}"),
                )
                .await;
            }
            return Err(mapped);
        }
    };

    let is_head = req.method() == actix_web::http::Method::HEAD;
    let mut resp = HttpResponse::Ok();
    resp.insert_header(("Docker-Content-Digest", handle.digest.clone()));
    resp.insert_header(("Content-Length", handle.size_bytes.to_string()));
    if let Some(mt) = &handle.media_type {
        resp.insert_header(("Content-Type", mt.clone()));
    }
    if is_head {
        return Ok(resp.finish());
    }

    let file = tokio::fs::File::open(&handle.path)
        .await
        .map_err(|_| OciError::Internal)?;
    let stream = FramedRead::new(file, BytesCodec::new()).map(|r| {
        r.map(|b: bytes::BytesMut| b.freeze())
            .map_err(|_| actix_web::error::ErrorInternalServerError("io"))
    });
    Ok(resp.streaming(stream))
}

/// Any non-GET/HEAD under /v2/* returns 405 per OCI "read-only" stance.
pub async fn push_not_supported() -> Result<HttpResponse, OciError> {
    Err(OciError::Unsupported)
}

fn map_reg_err(e: &RegistryError) -> OciError {
    match e {
        RegistryError::NotFound => OciError::ManifestUnknown,
        _ => OciError::Upstream,
    }
}

async fn audit_requested(
    pool: &PgPool,
    req: &HttpRequest,
    user: &OciBearerUser,
    app_id: &Uuid,
    reference: &str,
) {
    let log = CreateAuditLog::new(AuditAction::OciPullRequested)
        .with_actor(user.claims.sub, &user.email, &user.role)
        .with_ip(extract_client_ip(req).map(IpNetwork::from))
        .with_resource("application", *app_id)
        .with_metadata(serde_json::json!({ "reference": reference }));
    if let Err(e) = AuditLogRepository::create(pool, log).await {
        tracing::warn!(?e, "oci pull_requested audit log failed");
    }
}

async fn audit_completed(
    pool: &PgPool,
    req: &HttpRequest,
    user: &OciBearerUser,
    app_id: &Uuid,
    reference: &str,
    digest: &str,
) {
    let log = CreateAuditLog::new(AuditAction::OciPullCompleted)
        .with_actor(user.claims.sub, &user.email, &user.role)
        .with_ip(extract_client_ip(req).map(IpNetwork::from))
        .with_resource("application", *app_id)
        .with_metadata(serde_json::json!({ "reference": reference, "digest": digest }));
    if let Err(e) = AuditLogRepository::create(pool, log).await {
        tracing::warn!(?e, "oci pull_completed audit log failed");
    }
}

async fn audit_denied(
    pool: &PgPool,
    req: &HttpRequest,
    user: &OciBearerUser,
    app_id: &Uuid,
    reason: &str,
    reset_in_secs: Option<u64>,
) {
    let log = CreateAuditLog::new(AuditAction::OciPullDeniedRateLimit)
        .with_actor(user.claims.sub, &user.email, &user.role)
        .with_ip(extract_client_ip(req).map(IpNetwork::from))
        .with_resource("application", *app_id)
        .with_metadata(serde_json::json!({ "reason": reason, "reset_in_secs": reset_in_secs }));
    if let Err(e) = AuditLogRepository::create(pool, log).await {
        tracing::warn!(?e, "oci pull_denied audit log failed");
    }
}

async fn audit_denied_scope(
    pool: &PgPool,
    req: &HttpRequest,
    user: &OciBearerUser,
    requested_slug: &str,
) {
    let log = CreateAuditLog::new(AuditAction::OciPullDeniedScope)
        .with_actor(user.claims.sub, &user.email, &user.role)
        .with_ip(extract_client_ip(req).map(IpNetwork::from))
        .with_metadata(serde_json::json!({
            "requested_slug": requested_slug,
            "token_scope": user.claims.scope,
        }));
    if let Err(e) = AuditLogRepository::create(pool, log).await {
        tracing::warn!(?e, "oci pull_denied_scope audit log failed");
    }
}

async fn audit_failed_upstream(
    pool: &PgPool,
    req: &HttpRequest,
    user: &OciBearerUser,
    app_id: &Uuid,
    kind: &str,
    reference: &str,
    error: &str,
) {
    let log = CreateAuditLog::new(AuditAction::OciPullFailedUpstream)
        .with_actor(user.claims.sub, &user.email, &user.role)
        .with_ip(extract_client_ip(req).map(IpNetwork::from))
        .with_resource("application", *app_id)
        .with_metadata(serde_json::json!({
            "kind": kind,
            "reference": reference,
            "error": error,
        }));
    if let Err(e) = AuditLogRepository::create(pool, log).await {
        tracing::warn!(?e, "oci pull_failed_upstream audit log failed");
    }
}

#[cfg(test)]
mod integration {
    //! Full-stack pull: mock Forgejo registry + real Postgres via DATABASE_URL.
    //! Skipped automatically when DATABASE_URL is unset.

    use super::*;
    use actix_web::{http::header, test, App};
    use sha2::{Digest, Sha256};
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::services::oci_token::OciTokenService;
    use crate::services::JwtConfig;

    async fn maybe_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    async fn seed_user(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, subscription_status)
             VALUES ($1, $2, 'x', 'subscriber', 'active')",
        )
        .bind(id)
        .bind(format!("oci-integ-{}@example.com", id))
        .execute(pool)
        .await
        .unwrap();
        id
    }

    async fn seed_pullable_app(
        pool: &PgPool,
        owner: &str,
        name: &str,
        tag: &str,
    ) -> (Uuid, String) {
        let id = Uuid::new_v4();
        let slug = format!("app-{}", id.simple());
        sqlx::query(
            "INSERT INTO applications
             (id, name, slug, display_name, container_name, is_active,
              oci_image_owner, oci_image_name, pinned_image_tag)
             VALUES ($1, $2, $2, $3, $2, true, $4, $5, $6)",
        )
        .bind(id)
        .bind(&slug)
        .bind("Test App")
        .bind(owner)
        .bind(name)
        .bind(tag)
        .execute(pool)
        .await
        .unwrap();
        (id, slug)
    }

    async fn cleanup(pool: &PgPool, user_id: Uuid, app_id: Uuid, digest: &str) {
        sqlx::query("DELETE FROM oci_blob_cache WHERE content_digest = $1")
            .bind(digest)
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM applications WHERE id = $1")
            .bind(app_id)
            .execute(pool)
            .await
            .ok();
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
    }

    #[actix_rt::test]
    async fn happy_path_manifest_and_blob() {
        let Some(pool) = maybe_pool().await else {
            return;
        };

        // Upstream mock
        let server = MockServer::start().await;
        let manifest_body = br#"{"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"digest":"sha256:cfg","size":10},"layers":[]}"#.to_vec();
        let blob_body = format!("helloworld-{}", Uuid::new_v4()).into_bytes();
        let blob_digest = format!("sha256:{}", hex::encode(Sha256::digest(&blob_body)));

        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/manifests/.+"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(manifest_body.clone())
                    .insert_header("Content-Type", "application/vnd.oci.image.manifest.v1+json")
                    .insert_header("Docker-Content-Digest", "sha256:manifestdigest"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/blobs/.+"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(blob_body.clone())
                    .insert_header("Content-Type", "application/octet-stream"),
            )
            .mount(&server)
            .await;

        // Seed DB
        let user_id = seed_user(&pool).await;
        let (app_id, slug) = seed_pullable_app(&pool, "acme", "app", "v1").await;

        // Services
        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let manifest_cache = Arc::new(ManifestCache::new(300));
        let tmp = tempfile::tempdir().unwrap();
        let blob_cache = Arc::new(BlobCache::new(
            client.clone(),
            tmp.path().to_str().unwrap(),
            10_000_000,
            pool.clone(),
        ));
        blob_cache.ensure_dir().await.unwrap();
        let limiter = Arc::new(OciLimiter::new(2, 100));

        // Token
        let jwt_cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let token_svc = Arc::new(OciTokenService::new(&jwt_cfg, 900));
        let scope = format!("repository:{}:pull", slug);
        let token = token_svc.issue(user_id, &scope).unwrap();

        // Build in-process App mirroring the OCI App wiring
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(Some(client.clone())))
                .app_data(web::Data::new(Some(manifest_cache.clone())))
                .app_data(web::Data::new(Some(blob_cache.clone())))
                .app_data(web::Data::new(limiter.clone()))
                .app_data(token_svc.clone())
                .route(
                    "/v2/{slug}/manifests/{reference}",
                    web::get().to(get_manifest),
                )
                .route("/v2/{slug}/blobs/{digest}", web::get().to(get_blob)),
        )
        .await;

        // Manifest pull
        let req = test::TestRequest::get()
            .uri(&format!("/v2/{}/manifests/v1", slug))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "manifest status");
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        assert!(
            ct.contains("application/vnd.oci.image.manifest.v1+json"),
            "unexpected content-type: {ct}"
        );
        let body = test::read_body(resp).await;
        assert_eq!(body.as_ref(), manifest_body.as_slice());

        // Blob pull — request the digest that matches the bytes the mock will return
        let req = test::TestRequest::get()
            .uri(&format!("/v2/{}/blobs/{}", slug, blob_digest))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "blob status");
        let body = test::read_body(resp).await;
        assert_eq!(body.as_ref(), blob_body.as_slice());

        cleanup(&pool, user_id, app_id, &blob_digest).await;
    }

    fn build_app(
        pool: PgPool,
        client: Arc<ForgejoRegistryClient>,
        manifest_cache: Arc<ManifestCache>,
        blob_cache: Arc<BlobCache>,
        limiter: Arc<OciLimiter>,
        token_svc: Arc<OciTokenService>,
    ) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            .app_data(web::Data::new(pool))
            .app_data(web::Data::new(Some(client)))
            .app_data(web::Data::new(Some(manifest_cache)))
            .app_data(web::Data::new(Some(blob_cache)))
            .app_data(web::Data::new(limiter))
            .app_data(token_svc)
            .route(
                "/v2/{slug}/manifests/{reference}",
                web::get().to(get_manifest),
            )
            .route("/v2/{slug}/blobs/{digest}", web::get().to(get_blob))
            .route(
                "/v2/{slug}/blobs/uploads/",
                web::post().to(crate::handlers::oci_registry::push_not_supported),
            )
            .route(
                "/v2/{slug}/manifests/{reference}",
                web::put().to(crate::handlers::oci_registry::push_not_supported),
            )
    }

    #[actix_rt::test]
    async fn scope_mismatch_denies() {
        let Some(pool) = maybe_pool().await else {
            return;
        };

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/manifests/.+"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"{}".to_vec()))
            .mount(&server)
            .await;

        let user_id = seed_user(&pool).await;
        let (app_id, slug) = seed_pullable_app(&pool, "acme", "app", "v1").await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let manifest_cache = Arc::new(ManifestCache::new(300));
        let tmp = tempfile::tempdir().unwrap();
        let blob_cache = Arc::new(BlobCache::new(
            client.clone(),
            tmp.path().to_str().unwrap(),
            10_000_000,
            pool.clone(),
        ));
        blob_cache.ensure_dir().await.unwrap();
        let limiter = Arc::new(OciLimiter::new(2, 100));
        let jwt_cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let token_svc = Arc::new(OciTokenService::new(&jwt_cfg, 900));
        // Scope for a DIFFERENT slug
        let wrong_scope = "repository:some-other-slug:pull";
        let token = token_svc.issue(user_id, wrong_scope).unwrap();

        let app = test::init_service(build_app(
            pool.clone(),
            client,
            manifest_cache,
            blob_cache,
            limiter,
            token_svc,
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/v2/{}/manifests/v1", slug))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
        let body = test::read_body(resp).await;
        let s = String::from_utf8_lossy(&body);
        assert!(s.contains("DENIED"), "body should contain DENIED code: {s}");

        cleanup(&pool, user_id, app_id, "").await;
    }

    #[actix_rt::test]
    async fn cross_audience_token_rejected() {
        use crate::services::JwtService;
        let Some(pool) = maybe_pool().await else {
            return;
        };

        let server = MockServer::start().await;
        let user_id = seed_user(&pool).await;
        let (app_id, slug) = seed_pullable_app(&pool, "acme", "app", "v1").await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let manifest_cache = Arc::new(ManifestCache::new(300));
        let tmp = tempfile::tempdir().unwrap();
        let blob_cache = Arc::new(BlobCache::new(
            client.clone(),
            tmp.path().to_str().unwrap(),
            10_000_000,
            pool.clone(),
        ));
        blob_cache.ensure_dir().await.unwrap();
        let limiter = Arc::new(OciLimiter::new(2, 100));
        let jwt_cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let token_svc = Arc::new(OciTokenService::new(&jwt_cfg, 900));

        // Issue a primary-API access token (no aud="registry" claim)
        let user = crate::repositories::UserRepository::find_by_id(&pool, user_id)
            .await
            .unwrap()
            .unwrap();
        let jwt_svc = JwtService::new(jwt_cfg);
        let api_token = jwt_svc.create_access_token(&user).unwrap();

        let app = test::init_service(build_app(
            pool.clone(),
            client,
            manifest_cache,
            blob_cache,
            limiter,
            token_svc,
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/v2/{}/manifests/v1", slug))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", api_token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401, "cross-audience token must be rejected");

        cleanup(&pool, user_id, app_id, "").await;
    }

    #[actix_rt::test]
    async fn push_verbs_return_405() {
        let Some(pool) = maybe_pool().await else {
            return;
        };

        let server = MockServer::start().await;
        let user_id = seed_user(&pool).await;
        let (app_id, slug) = seed_pullable_app(&pool, "acme", "app", "v1").await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let manifest_cache = Arc::new(ManifestCache::new(300));
        let tmp = tempfile::tempdir().unwrap();
        let blob_cache = Arc::new(BlobCache::new(
            client.clone(),
            tmp.path().to_str().unwrap(),
            10_000_000,
            pool.clone(),
        ));
        blob_cache.ensure_dir().await.unwrap();
        let limiter = Arc::new(OciLimiter::new(2, 100));
        let jwt_cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let token_svc = Arc::new(OciTokenService::new(&jwt_cfg, 900));

        let app = test::init_service(build_app(
            pool.clone(),
            client,
            manifest_cache,
            blob_cache,
            limiter,
            token_svc,
        ))
        .await;

        for (method_name, req) in [
            (
                "POST uploads",
                test::TestRequest::post()
                    .uri(&format!("/v2/{}/blobs/uploads/", slug))
                    .to_request(),
            ),
            (
                "PUT manifest",
                test::TestRequest::put()
                    .uri(&format!("/v2/{}/manifests/v1", slug))
                    .to_request(),
            ),
        ] {
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 405, "{method_name} must return 405");
        }

        cleanup(&pool, user_id, app_id, "").await;
    }

    #[actix_rt::test]
    async fn manifest_cache_hit_skips_upstream() {
        let Some(pool) = maybe_pool().await else {
            return;
        };

        let server = MockServer::start().await;
        let manifest_body =
            br#"{"mediaType":"application/vnd.oci.image.manifest.v1+json"}"#.to_vec();
        // Expect exactly ONE upstream hit despite two pulls
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/manifests/.+"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(manifest_body.clone())
                    .insert_header("Content-Type", "application/vnd.oci.image.manifest.v1+json")
                    .insert_header("Docker-Content-Digest", "sha256:cachedman"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let user_id = seed_user(&pool).await;
        let (app_id, slug) = seed_pullable_app(&pool, "acme", "app", "v1").await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let manifest_cache = Arc::new(ManifestCache::new(300));
        let tmp = tempfile::tempdir().unwrap();
        let blob_cache = Arc::new(BlobCache::new(
            client.clone(),
            tmp.path().to_str().unwrap(),
            10_000_000,
            pool.clone(),
        ));
        blob_cache.ensure_dir().await.unwrap();
        let limiter = Arc::new(OciLimiter::new(2, 100));
        let jwt_cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let token_svc = Arc::new(OciTokenService::new(&jwt_cfg, 900));
        let token = token_svc
            .issue(user_id, &format!("repository:{}:pull", slug))
            .unwrap();

        let app = test::init_service(build_app(
            pool.clone(),
            client,
            manifest_cache,
            blob_cache,
            limiter,
            token_svc,
        ))
        .await;

        for _ in 0..2 {
            let req = test::TestRequest::get()
                .uri(&format!("/v2/{}/manifests/v1", slug))
                .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);
        }
        // wiremock's .expect(1) is verified on server drop — forcing a drop here is awkward.
        // Instead, verify by counting received requests.
        let count = server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.url.path().contains("/manifests/"))
            .count();
        assert_eq!(
            count, 1,
            "manifest cache must serve the 2nd pull without re-fetching"
        );

        cleanup(&pool, user_id, app_id, "").await;
    }

    #[actix_rt::test]
    async fn digest_mismatch_returns_502() {
        let Some(pool) = maybe_pool().await else {
            return;
        };

        let server = MockServer::start().await;
        let wrong_body = b"not-what-the-digest-says".to_vec();
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/blobs/.+"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(wrong_body))
            .mount(&server)
            .await;

        let user_id = seed_user(&pool).await;
        let (app_id, slug) = seed_pullable_app(&pool, "acme", "app", "v1").await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let manifest_cache = Arc::new(ManifestCache::new(300));
        let tmp = tempfile::tempdir().unwrap();
        let blob_cache = Arc::new(BlobCache::new(
            client.clone(),
            tmp.path().to_str().unwrap(),
            10_000_000,
            pool.clone(),
        ));
        blob_cache.ensure_dir().await.unwrap();
        let limiter = Arc::new(OciLimiter::new(2, 100));
        let jwt_cfg = JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n");
        let token_svc = Arc::new(OciTokenService::new(&jwt_cfg, 900));
        let token = token_svc
            .issue(user_id, &format!("repository:{}:pull", slug))
            .unwrap();

        let app = test::init_service(build_app(
            pool.clone(),
            client,
            manifest_cache,
            blob_cache,
            limiter,
            token_svc,
        ))
        .await;

        // A valid-looking digest whose bytes will NOT match upstream.
        let fake_digest = format!("sha256:{}", "a".repeat(64));
        let req = test::TestRequest::get()
            .uri(&format!("/v2/{}/blobs/{}", slug, fake_digest))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 502, "digest mismatch must return 502");

        // No row should have been inserted for this digest.
        let row = crate::repositories::OciBlobCacheRepository::find(&pool, &fake_digest)
            .await
            .unwrap();
        assert!(
            row.is_none(),
            "no blob cache row should exist on digest mismatch"
        );

        cleanup(&pool, user_id, app_id, &fake_digest).await;
    }
}
