//! OCI registry models: manifest descriptors, error envelope, DB rows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// One entry in the OCI error envelope.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OciErrorEntry {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

/// The full OCI error response body: `{"errors":[...]}`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OciErrorEnvelope {
    pub errors: Vec<OciErrorEntry>,
}

impl OciErrorEnvelope {
    pub fn single(code: &str, message: &str) -> Self {
        Self {
            errors: vec![OciErrorEntry {
                code: code.to_string(),
                message: message.to_string(),
                detail: None,
            }],
        }
    }
}

/// A manifest stored in the cache, plus bookkeeping needed to re-serve it.
#[derive(Debug, Clone)]
pub struct CachedManifest {
    pub bytes: bytes::Bytes,
    pub media_type: String,
    pub digest: String,
}

/// DB row for `oci_blob_cache`.
#[derive(Debug, Clone, FromRow)]
pub struct OciBlobCacheRow {
    pub content_digest: String,
    pub size_bytes: i64,
    pub media_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
}

/// Inputs for upserting a new cached blob.
#[derive(Debug, Clone)]
pub struct NewCachedBlob {
    pub content_digest: String,
    pub size_bytes: i64,
    pub media_type: Option<String>,
}

/// Parsed image-manifest body (the subset we care about for reachability sweep).
///
/// Covers both `application/vnd.oci.image.manifest.v1+json` and
/// `application/vnd.docker.distribution.manifest.v2+json` (same shape for
/// our purposes — config + layers, each with a digest).
#[derive(Debug, Clone, Deserialize)]
pub struct ParsedManifest {
    #[serde(default)]
    pub config: Option<ParsedDescriptor>,
    #[serde(default)]
    pub layers: Vec<ParsedDescriptor>,
    /// Present on an image index — child manifest descriptors.
    #[serde(default)]
    pub manifests: Vec<ParsedDescriptor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedDescriptor {
    pub digest: String,
    #[serde(default)]
    pub size: Option<i64>,
    #[serde(rename = "mediaType", default)]
    pub media_type: Option<String>,
}

impl ParsedManifest {
    /// Flatten all directly-referenced blob/manifest digests.
    pub fn referenced_digests(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(cfg) = &self.config {
            out.push(cfg.digest.clone());
        }
        for l in &self.layers {
            out.push(l.digest.clone());
        }
        for m in &self.manifests {
            out.push(m.digest.clone());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_envelope_serializes_to_spec_shape() {
        let env = OciErrorEnvelope::single("MANIFEST_UNKNOWN", "not here");
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "errors": [{"code": "MANIFEST_UNKNOWN", "message": "not here"}]
            })
        );
    }

    #[test]
    fn parsed_manifest_collects_image_digests() {
        let raw = serde_json::json!({
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {"digest": "sha256:aaa", "size": 10, "mediaType": "application/vnd.oci.image.config.v1+json"},
            "layers": [
                {"digest": "sha256:bbb", "size": 100},
                {"digest": "sha256:ccc", "size": 200}
            ]
        });
        let parsed: ParsedManifest = serde_json::from_value(raw).unwrap();
        let digests = parsed.referenced_digests();
        assert_eq!(digests, vec!["sha256:aaa", "sha256:bbb", "sha256:ccc"]);
    }

    #[test]
    fn parsed_manifest_collects_index_children() {
        let raw = serde_json::json!({
            "mediaType": "application/vnd.oci.image.index.v1+json",
            "manifests": [
                {"digest": "sha256:amd64", "size": 1000},
                {"digest": "sha256:arm64", "size": 1001}
            ]
        });
        let parsed: ParsedManifest = serde_json::from_value(raw).unwrap();
        let digests = parsed.referenced_digests();
        assert_eq!(digests, vec!["sha256:amd64", "sha256:arm64"]);
    }
}
