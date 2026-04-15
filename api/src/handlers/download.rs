//! Member and admin download handlers.

use actix_web::{http::header, web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use sqlx::PgPool;
use std::sync::Arc;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::errors::AppError;
use crate::middleware::{extract_client_ip, AdminUser, MemberUser};
use crate::models::download::{
    AppDownloadGroup, AppDownloadsResponse, DownloadAsset, ReleaseMetadata,
};
use crate::models::{AuditAction, CreateAuditLog};
use crate::repositories::{ApplicationRepository, AuditLogRepository};
use crate::responses::{get_request_id, success};
use crate::services::download_limiter::LimitDenial;
use crate::services::{DownloadCache, DownloadLimiter, ReleaseCache};

fn asset_href(slug: &str, asset_name: &str) -> String {
    format!(
        "/v1/applications/{}/downloads/{}",
        urlencoding::encode(slug),
        urlencoding::encode(asset_name),
    )
}

fn to_public_asset(a: &crate::models::download::ReleaseAsset, slug: &str) -> DownloadAsset {
    DownloadAsset {
        asset_name: a.name.clone(),
        size_bytes: a.size,
        content_type: a.content_type.clone(),
        download_url: asset_href(slug, &a.name),
    }
}

/// GET /v1/applications/{slug}/downloads
pub async fn list_app_downloads(
    req: HttpRequest,
    _user: MemberUser,
    pool: web::Data<PgPool>,
    release_cache: web::Data<Arc<ReleaseCache>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let slug = path.into_inner();
    let app = ApplicationRepository::find_active_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::not_found("Application"))?;

    if !app.is_downloadable() {
        return Ok(success(
            AppDownloadsResponse {
                release_tag: None,
                assets: vec![],
            },
            request_id,
        ));
    }
    let owner = app.forgejo_owner.as_deref().unwrap();
    let repo = app.forgejo_repo.as_deref().unwrap();
    let tag = app.pinned_release_tag.as_deref().unwrap();

    let release = fetch_release_or_502(&release_cache, app.id, owner, repo, tag).await?;
    let assets = release
        .assets
        .iter()
        .map(|a| to_public_asset(a, &app.slug))
        .collect();

    Ok(success(
        AppDownloadsResponse {
            release_tag: Some(release.tag_name.clone()),
            assets,
        },
        request_id,
    ))
}

/// GET /v1/downloads
pub async fn list_all_downloads(
    req: HttpRequest,
    _user: MemberUser,
    pool: web::Data<PgPool>,
    release_cache: web::Data<Arc<ReleaseCache>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let apps = ApplicationRepository::list_active(&pool).await?;

    let mut groups: Vec<AppDownloadGroup> = Vec::new();
    for app in apps {
        if !app.is_downloadable() {
            continue;
        }
        let owner = app.forgejo_owner.as_deref().unwrap();
        let repo = app.forgejo_repo.as_deref().unwrap();
        let tag = app.pinned_release_tag.as_deref().unwrap();
        // Best-effort: skip apps whose Forgejo call errors so one bad config
        // doesn't break the whole page.
        let release = match release_cache.get(app.id, owner, repo, tag).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(app = %app.slug, error = %e, "release fetch failed");
                continue;
            }
        };
        groups.push(AppDownloadGroup {
            app_slug: app.slug.clone(),
            app_display_name: app.display_name.clone(),
            icon_url: app.icon_url.clone(),
            release_tag: release.tag_name.clone(),
            assets: release
                .assets
                .iter()
                .map(|a| to_public_asset(a, &app.slug))
                .collect(),
        });
    }

    Ok(success(
        serde_json::json!({ "groups": groups }),
        request_id,
    ))
}

/// GET /v1/applications/{slug}/downloads/{asset_name}
pub async fn download_asset(
    req: HttpRequest,
    user: MemberUser,
    pool: web::Data<PgPool>,
    release_cache: web::Data<Arc<ReleaseCache>>,
    download_cache: web::Data<Arc<DownloadCache>>,
    limiter: web::Data<Arc<DownloadLimiter>>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (slug, asset_name) = path.into_inner();
    let ip = extract_client_ip(&req).map(ipnetwork::IpNetwork::from);

    let app = ApplicationRepository::find_active_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::not_found("Application"))?;
    if !app.is_downloadable() {
        return Err(AppError::not_found("Asset"));
    }

    // Rate limiting (before any upstream call).
    match limiter.acquire(&pool, user.0.sub).await? {
        Ok(guard) => {
            // Audit: requested.
            AuditLogRepository::create(
                &pool,
                CreateAuditLog::new(AuditAction::DownloadRequested)
                    .with_actor(user.0.sub, &user.0.email, &user.0.role)
                    .with_resource("application", app.id)
                    .with_ip(ip)
                    .with_metadata(serde_json::json!({
                        "slug": slug,
                        "asset_name": asset_name,
                    })),
            )
            .await?;

            let owner = app.forgejo_owner.as_deref().unwrap();
            let repo = app.forgejo_repo.as_deref().unwrap();
            let tag = app.pinned_release_tag.as_deref().unwrap();
            let release = fetch_release_or_502(&release_cache, app.id, owner, repo, tag).await?;

            let asset = release
                .assets
                .iter()
                .find(|a| a.name == asset_name)
                .ok_or(AppError::not_found("Asset"))?;

            let row = match download_cache.get_or_fetch(app.id, tag, asset).await {
                Ok(row) => row,
                Err(e) => {
                    AuditLogRepository::create(
                        &pool,
                        CreateAuditLog::new(AuditAction::DownloadFailedUpstream)
                            .with_actor(user.0.sub, &user.0.email, &user.0.role)
                            .with_resource("application", app.id)
                            .with_ip(ip)
                            .with_metadata(serde_json::json!({
                                "slug": slug,
                                "asset_name": asset_name,
                                "error": e.to_string(),
                            })),
                    )
                    .await?;
                    return Err(AppError::upstream("Download upstream failed"));
                }
            };

            let file_path = download_cache.file_path(&row.content_sha256);
            let file = tokio::fs::File::open(&file_path).await.map_err(|e| {
                tracing::error!(error = %e, "cached file vanished");
                AppError::internal("Cached file missing")
            })?;

            // Audit: completed (emitted before the response body is fully sent
            // — this is accurate for "request served from cache"; clients may
            // abort mid-stream, which our audit log intentionally does not
            // distinguish).
            AuditLogRepository::create(
                &pool,
                CreateAuditLog::new(AuditAction::DownloadCompleted)
                    .with_actor(user.0.sub, &user.0.email, &user.0.role)
                    .with_resource("application", app.id)
                    .with_ip(ip)
                    .with_metadata(serde_json::json!({
                        "slug": slug,
                        "asset_name": asset_name,
                        "size_bytes": row.size_bytes,
                    })),
            )
            .await?;

            // Attach the DownloadGuard to the stream so it drops only when the
            // stream is fully consumed or dropped — ensuring the concurrency
            // slot stays held for the duration of the streaming response.
            let framed = FramedRead::new(file, BytesCodec::new());
            let stream = futures_util::stream::unfold(
                (framed, guard),
                |(mut s, g)| async move {
                    match s.next().await {
                        Some(Ok(b)) => Some((Ok::<_, std::io::Error>(b.freeze()), (s, g))),
                        Some(Err(e)) => Some((Err(e), (s, g))),
                        None => {
                            drop(g);
                            None
                        }
                    }
                },
            );

            let resp = HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, row.content_type.clone()))
                .insert_header((header::CONTENT_LENGTH, row.size_bytes.to_string()))
                .insert_header((
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", asset_name),
                ))
                .streaming(stream);
            Ok(resp)
        }
        Err(LimitDenial::Concurrency) => {
            AuditLogRepository::create(
                &pool,
                CreateAuditLog::new(AuditAction::DownloadDeniedRateLimit)
                    .with_actor(user.0.sub, &user.0.email, &user.0.role)
                    .with_resource("application", app.id)
                    .with_ip(ip)
                    .with_metadata(serde_json::json!({
                        "slug": slug,
                        "asset_name": asset_name,
                        "reason": "concurrency",
                    })),
            )
            .await?;
            Err(AppError::rate_limited("download_concurrency_limit", None))
        }
        Err(LimitDenial::DailyCap { reset_in_secs }) => {
            AuditLogRepository::create(
                &pool,
                CreateAuditLog::new(AuditAction::DownloadDeniedRateLimit)
                    .with_actor(user.0.sub, &user.0.email, &user.0.role)
                    .with_resource("application", app.id)
                    .with_ip(ip)
                    .with_metadata(serde_json::json!({
                        "slug": slug,
                        "asset_name": asset_name,
                        "reason": "daily_cap",
                        "reset_in_secs": reset_in_secs,
                    })),
            )
            .await?;
            Err(AppError::rate_limited(
                "download_daily_limit",
                Some(reset_in_secs),
            ))
        }
    }
}

async fn fetch_release_or_502(
    cache: &ReleaseCache,
    app_id: uuid::Uuid,
    owner: &str,
    repo: &str,
    tag: &str,
) -> Result<Arc<ReleaseMetadata>, AppError> {
    cache.get(app_id, owner, repo, tag).await.map_err(|e| {
        tracing::warn!(error = %e, "forgejo release fetch failed");
        AppError::upstream("Forgejo upstream error")
    })
}

/// POST /v1/admin/applications/{slug}/downloads/refresh
pub async fn admin_refresh_release(
    req: HttpRequest,
    _admin: AdminUser,
    pool: web::Data<PgPool>,
    release_cache: web::Data<Arc<ReleaseCache>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let slug = path.into_inner();
    let app = ApplicationRepository::find_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::not_found("Application"))?;
    if !app.is_downloadable() {
        return Err(AppError::validation(
            "application",
            "Application is not configured for downloads",
        ));
    }
    let tag = app.pinned_release_tag.as_deref().unwrap();
    release_cache.invalidate(app.id, tag).await;
    let release = release_cache
        .get(
            app.id,
            app.forgejo_owner.as_deref().unwrap(),
            app.forgejo_repo.as_deref().unwrap(),
            tag,
        )
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "forgejo refresh failed");
            AppError::upstream("Forgejo upstream error")
        })?;

    let assets: Vec<_> = release
        .assets
        .iter()
        .map(|a| to_public_asset(a, &app.slug))
        .collect();
    Ok(success(
        AppDownloadsResponse {
            release_tag: Some(release.tag_name.clone()),
            assets,
        },
        request_id,
    ))
}
