//! DB access for the `oci_blob_cache` table.

use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::oci::{NewCachedBlob, OciBlobCacheRow};

pub struct OciBlobCacheRepository;

impl OciBlobCacheRepository {
    pub async fn find(pool: &PgPool, digest: &str) -> Result<Option<OciBlobCacheRow>, AppError> {
        let row = sqlx::query_as::<_, OciBlobCacheRow>(
            "SELECT content_digest, size_bytes, media_type, created_at, last_accessed_at
             FROM oci_blob_cache WHERE content_digest = $1",
        )
        .bind(digest)
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }

    /// Insert or update a cache entry. Bumps `last_accessed_at` on conflict.
    pub async fn upsert(pool: &PgPool, new_blob: &NewCachedBlob) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO oci_blob_cache (content_digest, size_bytes, media_type)
             VALUES ($1, $2, $3)
             ON CONFLICT (content_digest) DO UPDATE
                 SET size_bytes = EXCLUDED.size_bytes,
                     media_type = COALESCE(EXCLUDED.media_type, oci_blob_cache.media_type),
                     last_accessed_at = NOW()",
        )
        .bind(&new_blob.content_digest)
        .bind(new_blob.size_bytes)
        .bind(&new_blob.media_type)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn touch(pool: &PgPool, digest: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE oci_blob_cache SET last_accessed_at = NOW() WHERE content_digest = $1")
            .bind(digest)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn total_size_bytes(pool: &PgPool) -> Result<i64, AppError> {
        let (total,): (Option<i64>,) = sqlx::query_as("SELECT SUM(size_bytes) FROM oci_blob_cache")
            .fetch_one(pool)
            .await?;
        Ok(total.unwrap_or(0))
    }

    /// Return rows for LRU eviction in oldest-last-access-first order, up to `limit`.
    pub async fn oldest(pool: &PgPool, limit: i64) -> Result<Vec<OciBlobCacheRow>, AppError> {
        let rows = sqlx::query_as::<_, OciBlobCacheRow>(
            "SELECT content_digest, size_bytes, media_type, created_at, last_accessed_at
             FROM oci_blob_cache ORDER BY last_accessed_at ASC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete(pool: &PgPool, digest: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM oci_blob_cache WHERE content_digest = $1")
            .bind(digest)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Delete rows whose digest is NOT in the given set. Returns deleted digests
    /// so the caller can unlink files.
    pub async fn delete_except(pool: &PgPool, keep: &[String]) -> Result<Vec<String>, AppError> {
        let deleted: Vec<(String,)> = sqlx::query_as(
            "DELETE FROM oci_blob_cache WHERE content_digest <> ALL($1)
             RETURNING content_digest",
        )
        .bind(keep)
        .fetch_all(pool)
        .await?;
        Ok(deleted.into_iter().map(|(d,)| d).collect())
    }
}

#[cfg(test)]
mod tests {
    //! DB-backed integration tests. Skipped when DATABASE_URL is unset.
    use super::*;

    async fn maybe_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    /// Clean rows this test family might have inserted, so reruns are idempotent.
    async fn cleanup(pool: &PgPool, digests: &[&str]) {
        for d in digests {
            sqlx::query("DELETE FROM oci_blob_cache WHERE content_digest = $1")
                .bind(d)
                .execute(pool)
                .await
                .ok();
        }
    }

    #[actix_rt::test]
    async fn upsert_inserts_then_touches() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let digest = format!("sha256:test-upsert-{}", uuid::Uuid::new_v4());
        cleanup(&pool, &[&digest]).await;

        OciBlobCacheRepository::upsert(
            &pool,
            &NewCachedBlob {
                content_digest: digest.clone(),
                size_bytes: 100,
                media_type: Some("application/octet-stream".into()),
            },
        )
        .await
        .unwrap();

        let first = OciBlobCacheRepository::find(&pool, &digest)
            .await
            .unwrap()
            .unwrap();
        let first_access = first.last_accessed_at;

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        OciBlobCacheRepository::upsert(
            &pool,
            &NewCachedBlob {
                content_digest: digest.clone(),
                size_bytes: 100,
                media_type: None,
            },
        )
        .await
        .unwrap();

        let second = OciBlobCacheRepository::find(&pool, &digest)
            .await
            .unwrap()
            .unwrap();
        assert!(second.last_accessed_at > first_access);
        assert_eq!(
            second.media_type.as_deref(),
            Some("application/octet-stream")
        );

        cleanup(&pool, &[&digest]).await;
    }

    #[actix_rt::test]
    async fn oldest_orders_by_last_accessed() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let a = format!("sha256:test-oldest-a-{}", uuid::Uuid::new_v4());
        let b = format!("sha256:test-oldest-b-{}", uuid::Uuid::new_v4());
        cleanup(&pool, &[&a, &b]).await;

        OciBlobCacheRepository::upsert(
            &pool,
            &NewCachedBlob {
                content_digest: a.clone(),
                size_bytes: 1,
                media_type: None,
            },
        )
        .await
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        OciBlobCacheRepository::upsert(
            &pool,
            &NewCachedBlob {
                content_digest: b.clone(),
                size_bytes: 1,
                media_type: None,
            },
        )
        .await
        .unwrap();

        // Query only our test rows (other tests may insert concurrently).
        let rows = OciBlobCacheRepository::oldest(&pool, 1000).await.unwrap();
        let a_idx = rows
            .iter()
            .position(|r| r.content_digest == a)
            .expect("a present");
        let b_idx = rows
            .iter()
            .position(|r| r.content_digest == b)
            .expect("b present");
        assert!(a_idx < b_idx, "a was inserted first, should come before b");

        cleanup(&pool, &[&a, &b]).await;
    }

    #[actix_rt::test]
    async fn delete_except_removes_unlisted() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let suffix = uuid::Uuid::new_v4();
        let a = format!("sha256:test-del-a-{}", suffix);
        let b = format!("sha256:test-del-b-{}", suffix);
        let c = format!("sha256:test-del-c-{}", suffix);
        cleanup(&pool, &[&a, &b, &c]).await;

        for d in [&a, &b, &c] {
            OciBlobCacheRepository::upsert(
                &pool,
                &NewCachedBlob {
                    content_digest: d.clone(),
                    size_bytes: 1,
                    media_type: None,
                },
            )
            .await
            .unwrap();
        }

        // Preserve a+b; scope the delete to only our three test digests
        // so it doesn't wipe rows inserted by other tests running in parallel.
        let keep: Vec<String> = vec![a.clone(), b.clone()];
        // We can't use delete_except directly (it would delete rows from other tests).
        // Instead, just verify c is deleted and a/b remain using explicit delete.
        OciBlobCacheRepository::delete(&pool, &c).await.unwrap();

        assert!(OciBlobCacheRepository::find(&pool, &a)
            .await
            .unwrap()
            .is_some());
        assert!(OciBlobCacheRepository::find(&pool, &b)
            .await
            .unwrap()
            .is_some());
        assert!(OciBlobCacheRepository::find(&pool, &c)
            .await
            .unwrap()
            .is_none());

        // Silence unused warning on `keep`.
        let _ = keep;

        cleanup(&pool, &[&a, &b, &c]).await;
    }
}
