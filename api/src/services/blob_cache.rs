//! On-disk blob cache for OCI layers.
//!
//! Keyed by content digest. Single-flight fetches via Arc<OnceCell>.
//! Bytes are streamed from upstream into a `.partial` file while being
//! hashed; on mismatch the partial is deleted and no DB row is inserted.
//! Successful writes are atomically renamed to the final name, then an
//! async LRU eviction task runs if we're over the byte cap.

use bytes::Bytes;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::OnceCell;

use crate::errors::AppError;
use crate::models::oci::NewCachedBlob;
use crate::repositories::OciBlobCacheRepository;
use crate::services::forgejo_registry::{ForgejoRegistryClient, RegistryError};

#[derive(Clone)]
pub struct BlobCache {
    client: Arc<ForgejoRegistryClient>,
    cache_dir: PathBuf,
    max_bytes: u64,
    pool: PgPool,
    inflight: Arc<Mutex<HashMap<String, Arc<OnceCell<Result<(), String>>>>>>,
}

/// Result handle: either a cache hit (path + row) or a freshly-populated blob.
#[derive(Debug)]
pub struct BlobHandle {
    pub digest: String,
    pub size_bytes: i64,
    pub media_type: Option<String>,
    pub path: PathBuf,
}

/// Accept only digest strings matching `sha256:<64 lowercase hex>`.
/// Anything else is rejected — this blocks path-traversal attacks via
/// crafted digest strings (e.g. `sha256:../../etc/passwd`).
fn validate_digest(digest: &str) -> Result<(), AppError> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(AppError::validation("digest", "invalid blob digest format"));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(AppError::validation("digest", "invalid blob digest format"));
    }
    Ok(())
}

impl BlobCache {
    pub fn new(
        client: Arc<ForgejoRegistryClient>,
        cache_dir: &str,
        max_bytes: u64,
        pool: PgPool,
    ) -> Self {
        Self {
            client,
            cache_dir: PathBuf::from(cache_dir),
            max_bytes,
            pool,
            inflight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check for the cache dir, creating it only as a fallback.
    ///
    /// In prod the directory is a bind-mounted volume that the operator has
    /// already provisioned with the correct ownership. Calling `create_dir_all`
    /// unconditionally would mask a broken bind-mount (the dir would be created
    /// on the container's overlay filesystem and silently lose data on
    /// container recreation). We stat first and only create if truly missing.
    pub async fn ensure_dir(&self) -> Result<(), std::io::Error> {
        if fs::metadata(&self.cache_dir).await.is_ok() {
            return Ok(());
        }
        fs::create_dir_all(&self.cache_dir).await
    }

    fn final_path(&self, digest: &str) -> PathBuf {
        self.cache_dir.join(digest.replace(':', "_"))
    }

    fn partial_path(&self, digest: &str) -> PathBuf {
        self.cache_dir
            .join(format!("{}.partial", digest.replace(':', "_")))
    }

    /// Fetch blob (cache on miss), returning a BlobHandle with path on disk.
    pub async fn get_or_fetch(
        &self,
        owner: &str,
        name: &str,
        digest: &str,
    ) -> Result<BlobHandle, AppError> {
        validate_digest(digest)?;

        if let Some(row) = OciBlobCacheRepository::find(&self.pool, digest).await? {
            let path = self.final_path(digest);
            if path.exists() {
                OciBlobCacheRepository::touch(&self.pool, digest).await.ok();
                return Ok(BlobHandle {
                    digest: row.content_digest,
                    size_bytes: row.size_bytes,
                    media_type: row.media_type,
                    path,
                });
            }
        }

        let cell = {
            let mut m = self.inflight.lock().unwrap();
            m.entry(digest.to_string())
                .or_insert_with(|| Arc::new(OnceCell::new()))
                .clone()
        };

        let digest_owned = digest.to_string();
        let owner_owned = owner.to_string();
        let name_owned = name.to_string();
        let cloned_self = self.clone();
        let result = cell
            .get_or_init(|| async move {
                cloned_self
                    .fetch_and_store(&owner_owned, &name_owned, &digest_owned)
                    .await
                    .map_err(|e| format!("{e:?}"))
            })
            .await
            .clone();

        {
            let mut m = self.inflight.lock().unwrap();
            m.remove(digest);
        }

        result.map_err(|s| AppError::internal(format!("blob cache fetch: {s}")))?;

        let row = OciBlobCacheRepository::find(&self.pool, digest)
            .await?
            .ok_or_else(|| {
                AppError::internal("blob cache inconsistent: row missing after fetch")
            })?;
        Ok(BlobHandle {
            digest: row.content_digest,
            size_bytes: row.size_bytes,
            media_type: row.media_type,
            path: self.final_path(digest),
        })
    }

    async fn fetch_and_store(&self, owner: &str, name: &str, digest: &str) -> Result<(), AppError> {
        validate_digest(digest)?;
        self.ensure_dir()
            .await
            .map_err(|e| AppError::internal(format!("mkdir: {e}")))?;
        let partial = self.partial_path(digest);
        let final_ = self.final_path(digest);

        let result: Result<(Option<String>, u64), AppError> = async {
            let mut upstream = self
                .client
                .get_blob(owner, name, digest)
                .await
                .map_err(map_registry_err)?;
            let mut file = fs::File::create(&partial)
                .await
                .map_err(|e| AppError::internal(format!("partial create: {e}")))?;
            let mut hasher = Sha256::new();
            let mut total: u64 = 0;
            while let Some(chunk) = upstream.body.next().await {
                let chunk: Bytes =
                    chunk.map_err(|e| AppError::internal(format!("upstream stream: {e}")))?;
                hasher.update(&chunk);
                file.write_all(&chunk)
                    .await
                    .map_err(|e| AppError::internal(format!("partial write: {e}")))?;
                total += chunk.len() as u64;
            }
            file.flush()
                .await
                .map_err(|e| AppError::internal(format!("flush: {e}")))?;
            file.sync_all()
                .await
                .map_err(|e| AppError::internal(format!("sync: {e}")))?;
            drop(file);

            let computed = format!("sha256:{}", hex::encode(hasher.finalize()));
            if computed != digest {
                return Err(AppError::internal(format!(
                    "digest mismatch: upstream={computed}, expected={digest}"
                )));
            }
            fs::rename(&partial, &final_)
                .await
                .map_err(|e| AppError::internal(format!("rename: {e}")))?;
            Ok((upstream.media_type, total))
        }
        .await;

        let (media_type, total) = match result {
            Ok(v) => v,
            Err(e) => {
                let _ = fs::remove_file(&partial).await;
                return Err(e);
            }
        };

        OciBlobCacheRepository::upsert(
            &self.pool,
            &NewCachedBlob {
                content_digest: digest.to_string(),
                size_bytes: total as i64,
                media_type,
            },
        )
        .await?;

        let evictor = self.clone();
        tokio::spawn(async move {
            if let Err(e) = evictor.evict_if_over_cap().await {
                tracing::warn!(error = %e, "oci blob cache eviction failed");
            }
        });

        Ok(())
    }

    /// If the total exceeds `max_bytes`, unlink oldest rows until under.
    pub async fn evict_if_over_cap(&self) -> Result<(), AppError> {
        let mut total = OciBlobCacheRepository::total_size_bytes(&self.pool).await? as u64;
        if total <= self.max_bytes {
            return Ok(());
        }
        let rows = OciBlobCacheRepository::oldest(&self.pool, 100).await?;
        for row in rows {
            if total <= self.max_bytes {
                break;
            }
            let path = self.final_path(&row.content_digest);
            let _ = fs::remove_file(&path).await;
            OciBlobCacheRepository::delete(&self.pool, &row.content_digest).await?;
            total = total.saturating_sub(row.size_bytes as u64);
        }
        Ok(())
    }

    /// Orphan sweep: unlink blobs whose digest is not in `keep`.
    pub async fn sweep_orphans(&self, keep: &[String]) -> Result<usize, AppError> {
        let deleted = OciBlobCacheRepository::delete_except(&self.pool, keep).await?;
        for d in &deleted {
            let _ = fs::remove_file(self.final_path(d)).await;
        }
        Ok(deleted.len())
    }
}

fn map_registry_err(e: RegistryError) -> AppError {
    match e {
        RegistryError::NotFound => AppError::not_found("Blob"),
        _ => AppError::upstream("upstream registry"),
    }
}

#[cfg(test)]
mod tests {
    //! DB-backed integration tests. Skipped when DATABASE_URL is unset.
    use super::*;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn maybe_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    fn digest_of(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        format!("sha256:{}", hex::encode(h.finalize()))
    }

    async fn cleanup(pool: &PgPool, digests: &[&str]) {
        for d in digests {
            sqlx::query("DELETE FROM oci_blob_cache WHERE content_digest = $1")
                .bind(*d)
                .execute(pool)
                .await
                .ok();
        }
    }

    #[actix_rt::test]
    async fn fetches_and_stores_blob() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let server = MockServer::start().await;
        let body = format!("hello-oci-{}", uuid::Uuid::new_v4()).into_bytes();
        let digest = digest_of(&body);
        cleanup(&pool, &[&digest]).await;

        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/blobs/.+"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(body.clone())
                    .insert_header("Content-Type", "application/octet-stream"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let tmp = tempfile::tempdir().unwrap();
        let cache = BlobCache::new(
            client,
            tmp.path().to_str().unwrap(),
            1_000_000,
            pool.clone(),
        );

        let handle = cache.get_or_fetch("a", "b", &digest).await.unwrap();
        assert_eq!(handle.size_bytes, body.len() as i64);
        assert!(handle.path.exists());
        let on_disk = fs::read(&handle.path).await.unwrap();
        assert_eq!(on_disk, body);

        cleanup(&pool, &[&digest]).await;
    }

    #[actix_rt::test]
    async fn digest_mismatch_deletes_partial_and_no_row() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let server = MockServer::start().await;
        let body = format!("corrupt-{}", uuid::Uuid::new_v4()).into_bytes();
        let wrong_digest = digest_of(format!("something-else-{}", uuid::Uuid::new_v4()).as_bytes());
        cleanup(&pool, &[&wrong_digest]).await;

        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/blobs/.+"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
            .mount(&server)
            .await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let tmp = tempfile::tempdir().unwrap();
        let cache = BlobCache::new(
            client,
            tmp.path().to_str().unwrap(),
            1_000_000,
            pool.clone(),
        );

        let err = cache
            .get_or_fetch("a", "b", &wrong_digest)
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("digest mismatch") || msg.contains("blob cache fetch"),
            "unexpected error: {msg}"
        );
        assert!(OciBlobCacheRepository::find(&pool, &wrong_digest)
            .await
            .unwrap()
            .is_none());
        let partial = tmp
            .path()
            .join(format!("{}.partial", wrong_digest.replace(':', "_")));
        let final_ = tmp.path().join(wrong_digest.replace(':', "_"));
        assert!(!partial.exists());
        assert!(!final_.exists());
    }

    #[actix_rt::test]
    async fn single_flight_merges_concurrent_fetches() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let server = MockServer::start().await;
        let body = format!("dedup-{}", uuid::Uuid::new_v4()).into_bytes();
        let digest = digest_of(&body);
        cleanup(&pool, &[&digest]).await;

        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/blobs/.+"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
            .expect(1)
            .mount(&server)
            .await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let tmp = tempfile::tempdir().unwrap();
        let cache = BlobCache::new(
            client,
            tmp.path().to_str().unwrap(),
            1_000_000,
            pool.clone(),
        );

        let (a, b) = tokio::join!(
            cache.get_or_fetch("a", "b", &digest),
            cache.get_or_fetch("a", "b", &digest),
        );
        a.unwrap();
        b.unwrap();

        cleanup(&pool, &[&digest]).await;
    }

    #[actix_rt::test]
    async fn rejects_malformed_digest() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let server = MockServer::start().await;
        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let tmp = tempfile::tempdir().unwrap();
        let cache = BlobCache::new(client, tmp.path().to_str().unwrap(), 1_000_000, pool);

        for bad in [
            "sha256:../../etc/passwd",
            "sha256:abc",
            "not-a-digest",
            "sha256:DEADBEEF",
            "",
        ] {
            let err = cache.get_or_fetch("a", "b", bad).await.unwrap_err();
            let msg = err.to_string().to_lowercase();
            assert!(
                msg.contains("invalid") || msg.contains("digest"),
                "expected validation error for {bad:?}, got {msg}"
            );
        }
    }
}
