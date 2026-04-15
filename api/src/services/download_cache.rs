//! On-disk content-addressed cache for release assets.

use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, OnceCell};
use uuid::Uuid;

use crate::models::download::{DownloadCacheRow, ReleaseAsset};
use crate::repositories::DownloadCacheRepository;
use crate::services::forgejo::{ForgejoClient, ForgejoError};

#[derive(Debug, Error)]
pub enum DownloadCacheError {
    #[error("forgejo: {0}")]
    Forgejo(#[from] ForgejoError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("db: {0}")]
    Db(#[from] crate::errors::AppError),
    #[error("sha mismatch: expected {expected}, got {actual}")]
    ShaMismatch { expected: String, actual: String },
}

type CacheKey = (Uuid, String, String);
type InFlight = Arc<OnceCell<Result<Arc<DownloadCacheRow>, String>>>;

#[derive(Clone)]
pub struct DownloadCache {
    client: Arc<ForgejoClient>,
    cache_dir: PathBuf,
    max_bytes: u64,
    pool: PgPool,
    inflight: Arc<Mutex<HashMap<CacheKey, InFlight>>>,
}

impl DownloadCache {
    pub fn new(
        client: Arc<ForgejoClient>,
        cache_dir: impl Into<PathBuf>,
        max_bytes: u64,
        pool: PgPool,
    ) -> Self {
        Self {
            client,
            cache_dir: cache_dir.into(),
            max_bytes,
            pool,
            inflight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn file_path(&self, sha: &str) -> PathBuf {
        self.cache_dir.join(sha)
    }

    pub async fn ensure_dir(&self) -> Result<(), DownloadCacheError> {
        fs::create_dir_all(&self.cache_dir).await?;
        Ok(())
    }

    /// Return the cached row, fetching from Forgejo on miss.
    pub async fn get_or_fetch(
        &self,
        app_id: Uuid,
        release_tag: &str,
        asset: &ReleaseAsset,
    ) -> Result<Arc<DownloadCacheRow>, DownloadCacheError> {
        if let Some(row) = DownloadCacheRepository::find(
            &self.pool, app_id, release_tag, &asset.name,
        ).await? {
            let path = self.file_path(&row.content_sha256);
            if fs::metadata(&path).await.is_ok() {
                DownloadCacheRepository::touch(&self.pool, row.id).await?;
                return Ok(Arc::new(row));
            }
        }

        let key: CacheKey = (app_id, release_tag.to_string(), asset.name.clone());

        let cell = {
            let mut m = self.inflight.lock().await;
            m.entry(key.clone())
                .or_insert_with(|| Arc::new(OnceCell::new()))
                .clone()
        };

        let result = cell
            .get_or_init(|| async {
                match self.fetch_and_store(app_id, release_tag, asset).await {
                    Ok(row) => Ok(Arc::new(row)),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await
            .clone();

        {
            let mut m = self.inflight.lock().await;
            m.remove(&key);
        }

        match result {
            Ok(row) => Ok(row),
            Err(msg) => Err(DownloadCacheError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                msg,
            ))),
        }
    }

    async fn fetch_and_store(
        &self,
        app_id: Uuid,
        release_tag: &str,
        asset: &ReleaseAsset,
    ) -> Result<DownloadCacheRow, DownloadCacheError> {
        self.ensure_dir().await?;

        let tmp_name = format!(".tmp-{}", Uuid::new_v4());
        let tmp_path = self.cache_dir.join(&tmp_name);
        let mut file = fs::File::create(&tmp_path).await?;
        let mut hasher = Sha256::new();
        let mut total: i64 = 0;

        let mut stream = self.client.download_asset(&asset.browser_download_url).await?;
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| {
                DownloadCacheError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
            hasher.update(&bytes);
            file.write_all(&bytes).await?;
            total += bytes.len() as i64;
        }
        file.flush().await?;
        file.sync_all().await?;
        drop(file);

        let sha = hex::encode(hasher.finalize());
        let final_path = self.file_path(&sha);
        fs::rename(&tmp_path, &final_path).await?;

        let row = DownloadCacheRepository::upsert(
            &self.pool,
            app_id,
            release_tag,
            &asset.name,
            &sha,
            total,
            &asset.content_type,
        ).await?;

        let pool = self.pool.clone();
        let dir = self.cache_dir.clone();
        let cap = self.max_bytes;
        tokio::spawn(async move {
            if let Err(e) = evict_lru_to_fit(&pool, &dir, cap).await {
                tracing::warn!(error = %e, "download cache eviction failed");
            }
        });

        Ok(row)
    }

    pub async fn invalidate_app_tag(
        &self,
        app_id: Uuid,
        release_tag: &str,
    ) -> Result<(), DownloadCacheError> {
        let shas = DownloadCacheRepository::delete_for_tag(&self.pool, app_id, release_tag).await?;
        for sha in shas {
            if !DownloadCacheRepository::sha_referenced(&self.pool, &sha).await? {
                let path = self.file_path(&sha);
                let _ = fs::remove_file(&path).await;
            }
        }
        Ok(())
    }
}

async fn evict_lru_to_fit(
    pool: &PgPool,
    cache_dir: &Path,
    max_bytes: u64,
) -> Result<(), DownloadCacheError> {
    loop {
        let total = DownloadCacheRepository::total_bytes(pool).await?;
        if (total as u64) <= max_bytes {
            break;
        }
        let rows = DownloadCacheRepository::list_lru(pool, 32).await?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            DownloadCacheRepository::delete_by_id(pool, row.id).await?;
            if !DownloadCacheRepository::sha_referenced(pool, &row.content_sha256).await? {
                let path = cache_dir.join(&row.content_sha256);
                let _ = fs::remove_file(&path).await;
            }
            let total = DownloadCacheRepository::total_bytes(pool).await?;
            if (total as u64) <= max_bytes {
                return Ok(());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn file_path_joins_sha() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = DownloadCache::new(
            Arc::new(crate::services::forgejo::ForgejoClient::new(
                "http://unused".into(),
                "t".into(),
            )),
            tmp.path(),
            1024,
            PgPool::connect_lazy("postgres://u:p@127.0.0.1/db").unwrap(),
        );
        let p = cache.file_path("abc");
        assert!(p.ends_with("abc"));
    }
}
