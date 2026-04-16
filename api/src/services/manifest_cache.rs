//! TTL cache for manifest bytes + media type + digest.
//!
//! Keyed by `(app_id, reference)` where reference is either a tag or a
//! digest. Invalidated on admin tag change.

use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::models::oci::CachedManifest;

#[derive(Clone)]
pub struct ManifestCache {
    cache: Cache<(Uuid, String), Arc<CachedManifest>>,
}

impl ManifestCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(ttl_secs))
                .max_capacity(1024)
                .build(),
        }
    }

    pub async fn get(&self, app_id: Uuid, reference: &str) -> Option<Arc<CachedManifest>> {
        self.cache.get(&(app_id, reference.to_string())).await
    }

    pub async fn insert(&self, app_id: Uuid, reference: &str, value: CachedManifest) -> Arc<CachedManifest> {
        let arc = Arc::new(value);
        self.cache.insert((app_id, reference.to_string()), arc.clone()).await;
        arc
    }

    /// Invalidate every entry for the given application.
    pub async fn invalidate_app(&self, app_id: Uuid) {
        // moka doesn't support range invalidation; iterate and invalidate each key.
        let keys: Vec<_> = self
            .cache
            .iter()
            .filter_map(|(key, _)| {
                if key.0 == app_id {
                    Some((*key).clone())
                } else {
                    None
                }
            })
            .collect();
        for key in keys {
            self.cache.invalidate(&key).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn fixture() -> CachedManifest {
        CachedManifest {
            bytes: Bytes::from_static(b"{}"),
            media_type: "application/vnd.oci.image.manifest.v1+json".into(),
            digest: "sha256:abc".into(),
        }
    }

    #[actix_rt::test]
    async fn insert_then_get_returns_entry() {
        let cache = ManifestCache::new(60);
        let app = Uuid::new_v4();
        cache.insert(app, "v1", fixture()).await;
        let got = cache.get(app, "v1").await.unwrap();
        assert_eq!(got.digest, "sha256:abc");
    }

    #[actix_rt::test]
    async fn invalidate_app_removes_all_its_refs() {
        let cache = ManifestCache::new(60);
        let app = Uuid::new_v4();
        let other = Uuid::new_v4();
        cache.insert(app, "v1", fixture()).await;
        cache.insert(app, "v2", fixture()).await;
        cache.insert(other, "v1", fixture()).await;

        cache.invalidate_app(app).await;

        assert!(cache.get(app, "v1").await.is_none());
        assert!(cache.get(app, "v2").await.is_none());
        assert!(cache.get(other, "v1").await.is_some());
    }
}
