# OCI Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up an OCI Distribution Spec v1.1 compliant read-only registry at `registry.example.com` that proxies per-application pinned container images from Forgejo, authenticating via the OCI Bearer flow against existing a8n credentials + active-membership checks.

**Architecture:** A second `HttpServer` in the existing `a8n-api` binary, bound to an internal `OCI_REGISTRY_PORT` (default `18081`), exposes `/v2/*` and `/auth/token`. New services: `ForgejoRegistryClient` + `ManifestCache` + `BlobCache` + `OciTokenService` + `OciLimiter`. Three new migrations add OCI fields on `applications` plus `oci_blob_cache` and `oci_pull_daily_counts` tables. Admin UI gains three form fields. Bearer tokens reuse the existing JWT keypair with `aud="registry"`. Blob cache mirrors the existing download cache: SHA-256 addressed, single-flight, atomic rename + fsync, async LRU eviction.

**Tech Stack:** Rust + Actix-Web + sqlx (Postgres) + reqwest + tokio streams + moka + sha2; `wiremock` for Forgejo HTTP stubbing in integration tests. Frontend: React + TypeScript (admin form field changes only).

**Reference:** Design spec at `docs/superpowers/specs/2026-04-16-oci-registry-design.md`.

**Repo conventions (read before starting):**
- All `cargo` / `bun` commands run inside their containers via `just`. Use `just test-api`, `just test-frontend`, `just migrate`. To run a single Rust test: `docker compose -f compose.dev.yml exec api cargo test <name> --lib -- --nocapture`.
- Migrations live in `api/migrations/` with prefix `YYYYMMDDNNNNNN_description.sql`. The highest existing prefix is `20260415000039`. New migrations in this plan use `20260416000040..42`.
- Rust tests use in-module `#[cfg(test)] mod tests { ... }` blocks, not a separate `tests/` directory.
- API responses wrap payloads via helpers in `crate::responses`; errors use `AppError` in `crate::errors::AppError`. Registry endpoints are an exception: they use the OCI error envelope via a new `OciError` type (Task 9).
- Audit logs: `CreateAuditLog::new(AuditAction::...).with_actor(...).with_resource(...).with_metadata(...)` then `AuditLogRepository::create(pool, log).await?`.
- Conventional commits: `feat(oci-registry): ...`, `test(oci-registry): ...`, `docs(oci-registry): ...`.
- Don't break existing download proxy code — it shares `ForgejoClient` config (`FORGEJO_BASE_URL` / `FORGEJO_API_TOKEN`) but uses different caches and endpoints.

---

## File Structure

### New Rust files
- `api/migrations/20260416000040_add_oci_columns_to_applications.sql`
- `api/migrations/20260416000041_create_oci_blob_cache.sql`
- `api/migrations/20260416000042_create_oci_pull_daily_counts.sql`
- `api/src/models/oci.rs` — manifest descriptor, error envelope, DB row types
- `api/src/repositories/oci_blob_cache.rs` — CRUD for `oci_blob_cache`
- `api/src/repositories/oci_pull_daily_counts.rs` — daily-counter upsert
- `api/src/services/forgejo_registry.rs` — `/v2/*` client with URL host validation
- `api/src/services/manifest_cache.rs` — moka TTL cache
- `api/src/services/blob_cache.rs` — on-disk SHA-256 addressed cache + single-flight
- `api/src/services/oci_token.rs` — registry JWT issuer / verifier
- `api/src/services/oci_limiter.rs` — per-user manifest concurrency + daily counter
- `api/src/middleware/oci_auth.rs` — `OciBearerUser` extractor
- `api/src/errors/oci.rs` — `OciError` enum + `ResponseError` impl
- `api/src/handlers/oci_registry.rs` — `/v2/*` handlers
- `api/src/handlers/oci_auth.rs` — `/auth/token` handler
- `api/src/handlers/admin_oci.rs` — `POST /v1/admin/applications/{slug}/oci/refresh`
- `api/src/routes/oci.rs` — registry route config + `build_oci_app` factory

### Modified Rust files
- `api/src/models/application.rs` — three new fields on `Application` + `UpdateApplication`
- `api/src/models/mod.rs` — export `oci`
- `api/src/models/audit.rs` — six new `AuditAction` variants + `as_str`
- `api/src/repositories/application.rs` — `update` accepts OCI fields
- `api/src/repositories/mod.rs` — export new repos
- `api/src/services/mod.rs` — export new services
- `api/src/handlers/mod.rs` — export new handlers
- `api/src/handlers/admin.rs` — `update_application` accepts OCI fields + invalidates caches
- `api/src/routes/mod.rs` — register `admin_oci::refresh` under admin
- `api/src/errors/mod.rs` — export `OciError`
- `api/src/errors.rs` — no change (AppError stays separate)
- `api/src/middleware/mod.rs` — export `OciBearerUser`
- `api/src/config.rs` — new `OciConfig` + env loading on `Config`
- `api/src/main.rs` — initialize OCI services; spawn second `HttpServer`; `tokio::try_join!` both

### New frontend files (tests only — form fields are small inline changes)
- none new; tests extend `AdminApplicationsPage.test.tsx`

### Modified frontend files
- `frontend/src/types/index.ts` — add `oci_image_owner`, `oci_image_name`, `pinned_image_tag` to `Application`
- `frontend/src/pages/admin/AdminApplicationsPage.tsx` + test — three new form fields
- `frontend/src/api/admin.ts` (or wherever `updateApplication` lives) — include new fields in payload

### Ops
- `compose.yml`, `compose.dev.yml` — new named volume `oci_cache` → `/var/cache/a8n-oci`; plumb eight new env vars; expose port `18081`

### Docs
- `docs/oci-registry.md` — user + dev documentation
- `CLAUDE.md` — feature-flag note under "Feature Flags"

---

## Task 1: Migration — OCI columns on `applications`

**Files:**
- Create: `api/migrations/20260416000040_add_oci_columns_to_applications.sql`

- [ ] **Step 1: Write the migration**

```sql
-- Add OCI registry proxy configuration to applications.
-- When all three columns are non-null, the application is "pullable".

ALTER TABLE applications
    ADD COLUMN oci_image_owner    TEXT,
    ADD COLUMN oci_image_name     TEXT,
    ADD COLUMN pinned_image_tag   TEXT;

CREATE INDEX applications_pullable_idx
    ON applications (id)
    WHERE oci_image_owner IS NOT NULL
      AND oci_image_name IS NOT NULL
      AND pinned_image_tag IS NOT NULL;
```

- [ ] **Step 2: Run the migration**

Run: `just migrate`
Expected: success log, no errors. Verify with `just db-shell` → `\d applications` → three new columns listed.

- [ ] **Step 3: Commit**

```bash
git add api/migrations/20260416000040_add_oci_columns_to_applications.sql
git commit -m "feat(oci-registry): add oci image columns to applications"
```

---

## Task 2: Migration — `oci_blob_cache` table

**Files:**
- Create: `api/migrations/20260416000041_create_oci_blob_cache.sql`

- [ ] **Step 1: Write the migration**

```sql
-- Tracks every OCI blob we have cached on disk.
-- Filename on disk = content_digest (full sha256:<hex> string stored).
CREATE TABLE oci_blob_cache (
    content_digest    TEXT PRIMARY KEY,
    size_bytes        BIGINT NOT NULL,
    media_type        TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX oci_blob_cache_lru_idx ON oci_blob_cache (last_accessed_at);
```

- [ ] **Step 2: Run the migration**

Run: `just migrate`
Expected: success log. `\d oci_blob_cache` shows all five columns + LRU index.

- [ ] **Step 3: Commit**

```bash
git add api/migrations/20260416000041_create_oci_blob_cache.sql
git commit -m "feat(oci-registry): create oci_blob_cache table"
```

---

## Task 3: Migration — `oci_pull_daily_counts` table

**Files:**
- Create: `api/migrations/20260416000042_create_oci_pull_daily_counts.sql`

- [ ] **Step 1: Write the migration**

```sql
-- Per-user daily pull counter. UTC day boundary.
-- Counted once per manifest pull; blob fetches don't increment.
CREATE TABLE oci_pull_daily_counts (
    user_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    day_utc  DATE NOT NULL,
    count    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, day_utc)
);
```

- [ ] **Step 2: Run the migration**

Run: `just migrate`
Expected: success log. `\d oci_pull_daily_counts` shows the three columns.

- [ ] **Step 3: Commit**

```bash
git add api/migrations/20260416000042_create_oci_pull_daily_counts.sql
git commit -m "feat(oci-registry): create oci_pull_daily_counts table"
```

---

## Task 4: Extend `Application` model with OCI fields

**Files:**
- Modify: `api/src/models/application.rs`

- [ ] **Step 1: Add fields to `Application` struct**

Find the struct (grep for `pub struct Application`) and add three fields next to the existing `forgejo_*` fields:

```rust
pub oci_image_owner: Option<String>,
pub oci_image_name: Option<String>,
pub pinned_image_tag: Option<String>,
```

Find `UpdateApplication` in the same file and add the same three fields (all `Option<Option<String>>` if the struct uses that pattern for nullable updates — copy the pattern used by `forgejo_owner`).

- [ ] **Step 2: Add `is_pullable()` helper**

Add an impl block method on `Application`:

```rust
impl Application {
    /// True when all three OCI fields are set AND the application is active.
    pub fn is_pullable(&self) -> bool {
        self.is_active
            && self.oci_image_owner.is_some()
            && self.oci_image_name.is_some()
            && self.pinned_image_tag.is_some()
    }
}
```

(If an existing `impl Application` block exists, add the method inside it.)

- [ ] **Step 3: Write the test**

Append to the `#[cfg(test)] mod tests` block at the bottom of the file:

```rust
#[test]
fn is_pullable_requires_all_three_oci_fields_and_active() {
    let base = Application {
        is_active: true,
        oci_image_owner: Some("a8n".into()),
        oci_image_name: Some("rus".into()),
        pinned_image_tag: Some("v1.0".into()),
        ..Application::test_default()
    };
    assert!(base.is_pullable());

    let mut inactive = base.clone();
    inactive.is_active = false;
    assert!(!inactive.is_pullable());

    let mut no_tag = base.clone();
    no_tag.pinned_image_tag = None;
    assert!(!no_tag.is_pullable());
}
```

If `Application::test_default()` doesn't exist, add one in a `#[cfg(test)]` impl block that returns a minimally-populated `Application` with all required fields filled with placeholder values.

- [ ] **Step 4: Run the test**

Run: `docker compose -f compose.dev.yml exec api cargo test is_pullable --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add api/src/models/application.rs
git commit -m "feat(oci-registry): add oci fields to Application model"
```

---

## Task 5: Extend `ApplicationRepository::update` to accept OCI fields

**Files:**
- Modify: `api/src/repositories/application.rs`

- [ ] **Step 1: Update the SQL in `update()`**

Find the `update` method. Extend the `UPDATE applications SET …` statement to also set the three new columns, using `COALESCE(... existing column)` for no-change semantics (mirror the forgejo_* handling already present). Add the three new bindings in order, matching the `UpdateApplication` struct fields.

Exact shape (the statement is big — only these lines are added):

```rust
// inside the UPDATE ... SET clause, adjacent to forgejo_* lines:
oci_image_owner     = COALESCE($N::TEXT,     oci_image_owner),
oci_image_name      = COALESCE($N+1::TEXT,   oci_image_name),
pinned_image_tag    = COALESCE($N+2::TEXT,   pinned_image_tag),
```

(N is the next positional parameter after the existing ones; update both the SQL and the `.bind(...)` chain.)

Also update the `SELECT ... FROM applications` read queries in the same file (`find_active_by_slug`, `list_active`, etc.) to select the three new columns.

- [ ] **Step 2: Write the test**

Append to the existing test module in the repository file:

```rust
#[sqlx::test]
async fn update_sets_oci_fields(pool: PgPool) {
    let app = ApplicationRepository::create(&pool, CreateApplication {
        slug: "pull-test".into(),
        display_name: "Pull Test".into(),
        // ... other required fields per the struct
    }).await.unwrap();

    ApplicationRepository::update(&pool, app.id, UpdateApplication {
        oci_image_owner: Some(Some("a8n".into())),
        oci_image_name: Some(Some("rus".into())),
        pinned_image_tag: Some(Some("v1".into())),
        ..UpdateApplication::empty()
    }).await.unwrap();

    let reloaded = ApplicationRepository::find_active_by_slug(&pool, "pull-test")
        .await.unwrap().unwrap();
    assert_eq!(reloaded.oci_image_owner.as_deref(), Some("a8n"));
    assert_eq!(reloaded.oci_image_name.as_deref(), Some("rus"));
    assert_eq!(reloaded.pinned_image_tag.as_deref(), Some("v1"));
}
```

(Fill in `CreateApplication` required fields and match the pattern in the existing `UpdateApplication` tests for `forgejo_*`.)

- [ ] **Step 3: Run the test**

Run: `docker compose -f compose.dev.yml exec api cargo test update_sets_oci_fields --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/repositories/application.rs
git commit -m "feat(oci-registry): extend ApplicationRepository to update oci fields"
```

---

## Task 6: New `AuditAction` variants

**Files:**
- Modify: `api/src/models/audit.rs`

- [ ] **Step 1: Add the variants**

Add after the existing `DownloadFailedUpstream` variant:

```rust
OciLoginSucceeded,
OciLoginFailed,
OciPullRequested,
OciPullCompleted,
OciPullFailedUpstream,
OciPullDeniedRateLimit,
```

- [ ] **Step 2: Add matching `as_str` arms**

In the `as_str` match block, add (lowercase snake_case):

```rust
AuditAction::OciLoginSucceeded     => "oci_login_succeeded",
AuditAction::OciLoginFailed        => "oci_login_failed",
AuditAction::OciPullRequested      => "oci_pull_requested",
AuditAction::OciPullCompleted      => "oci_pull_completed",
AuditAction::OciPullFailedUpstream => "oci_pull_failed_upstream",
AuditAction::OciPullDeniedRateLimit => "oci_pull_denied_rate_limit",
```

No additions to `is_admin_action` (these are member actions).

- [ ] **Step 3: Write tests**

Append to the existing `#[cfg(test)] mod tests { ... }` block:

```rust
#[test]
fn audit_action_oci_variants() {
    assert_eq!(AuditAction::OciLoginSucceeded.as_str(), "oci_login_succeeded");
    assert_eq!(AuditAction::OciLoginFailed.as_str(), "oci_login_failed");
    assert_eq!(AuditAction::OciPullRequested.as_str(), "oci_pull_requested");
    assert_eq!(AuditAction::OciPullCompleted.as_str(), "oci_pull_completed");
    assert_eq!(AuditAction::OciPullFailedUpstream.as_str(), "oci_pull_failed_upstream");
    assert_eq!(AuditAction::OciPullDeniedRateLimit.as_str(), "oci_pull_denied_rate_limit");

    assert!(!AuditAction::OciPullRequested.is_admin_action());
    assert!(!AuditAction::OciLoginFailed.is_admin_action());
}
```

- [ ] **Step 4: Run the test**

Run: `docker compose -f compose.dev.yml exec api cargo test audit_action_oci_variants --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add api/src/models/audit.rs
git commit -m "feat(oci-registry): add oci audit action variants"
```

---

## Task 7: Config — `OciConfig`

**Files:**
- Modify: `api/src/config.rs`

- [ ] **Step 1: Add `OciConfig` struct and `from_env`**

Below the existing `DownloadConfig`:

```rust
/// OCI registry configuration.
#[derive(Debug, Clone)]
pub struct OciConfig {
    pub enabled: bool,
    pub port: u16,
    pub service: String,
    pub blob_cache_dir: String,
    pub blob_cache_max_bytes: u64,
    pub manifest_cache_ttl_secs: u64,
    pub concurrent_manifests_per_user: u32,
    pub pulls_per_user_per_day: u32,
    pub token_ttl_secs: u64,
}

impl OciConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("OCI_REGISTRY_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            port: env::var("OCI_REGISTRY_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(18081),
            service: env::var("OCI_REGISTRY_SERVICE")
                .unwrap_or_else(|_| "registry.example.com".to_string()),
            blob_cache_dir: env::var("OCI_BLOB_CACHE_DIR")
                .unwrap_or_else(|_| "/var/cache/a8n-oci".to_string()),
            blob_cache_max_bytes: env::var("OCI_BLOB_CACHE_MAX_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(53_687_091_200), // 50 GiB
            manifest_cache_ttl_secs: env::var("OCI_MANIFEST_CACHE_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            concurrent_manifests_per_user: env::var("OCI_CONCURRENT_MANIFESTS_PER_USER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            pulls_per_user_per_day: env::var("OCI_PULLS_PER_USER_PER_DAY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            token_ttl_secs: env::var("OCI_TOKEN_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900),
        }
    }
}
```

- [ ] **Step 2: Wire into `Config`**

Add field on the `Config` struct:

```rust
pub oci: OciConfig,
```

In `Config::from_env()`:

```rust
let oci = OciConfig::from_env();
```

Then include `oci,` in the final struct-literal return.

- [ ] **Step 3: Write tests**

Append to the `#[cfg(test)] mod tests` block:

```rust
#[test]
fn oci_config_defaults() {
    env::remove_var("OCI_REGISTRY_ENABLED");
    env::remove_var("OCI_REGISTRY_PORT");
    env::remove_var("OCI_REGISTRY_SERVICE");
    env::remove_var("OCI_BLOB_CACHE_DIR");
    env::remove_var("OCI_BLOB_CACHE_MAX_BYTES");
    env::remove_var("OCI_MANIFEST_CACHE_TTL_SECS");
    env::remove_var("OCI_CONCURRENT_MANIFESTS_PER_USER");
    env::remove_var("OCI_PULLS_PER_USER_PER_DAY");
    env::remove_var("OCI_TOKEN_TTL_SECS");

    let cfg = OciConfig::from_env();
    assert!(!cfg.enabled);
    assert_eq!(cfg.port, 18081);
    assert_eq!(cfg.blob_cache_dir, "/var/cache/a8n-oci");
    assert_eq!(cfg.blob_cache_max_bytes, 53_687_091_200);
    assert_eq!(cfg.manifest_cache_ttl_secs, 300);
    assert_eq!(cfg.concurrent_manifests_per_user, 2);
    assert_eq!(cfg.pulls_per_user_per_day, 50);
    assert_eq!(cfg.token_ttl_secs, 900);
}

#[test]
fn oci_config_enabled_when_set() {
    env::set_var("OCI_REGISTRY_ENABLED", "true");
    let cfg = OciConfig::from_env();
    assert!(cfg.enabled);
    env::remove_var("OCI_REGISTRY_ENABLED");
}
```

- [ ] **Step 4: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test oci_config --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add api/src/config.rs
git commit -m "feat(oci-registry): add OciConfig env loader"
```

---

## Task 8: `oci` model module — manifest descriptor + error envelope

**Files:**
- Create: `api/src/models/oci.rs`
- Modify: `api/src/models/mod.rs`

- [ ] **Step 1: Write the module**

```rust
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
```

- [ ] **Step 2: Export from `models/mod.rs`**

Add:

```rust
pub mod oci;
```

- [ ] **Step 3: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test oci::tests --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/models/oci.rs api/src/models/mod.rs
git commit -m "feat(oci-registry): add oci model module"
```

---

## Task 9: `OciError` enum + `ResponseError` impl

**Files:**
- Create: `api/src/errors/oci.rs`
- Modify: `api/src/errors/mod.rs` (export `oci`) or `api/src/errors.rs` (if errors is a single file)

> If `api/src/errors` is currently a single file (`errors.rs`), convert it by creating `api/src/errors/mod.rs` that re-exports everything from a new `api/src/errors/core.rs` (move the existing content), then add `pub mod oci; pub use oci::OciError;`. If already a directory, skip the conversion and add the module line.

- [ ] **Step 1: Create the error module**

```rust
//! OCI-flavored error type. Uses the OCI error envelope on the wire.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

use crate::models::oci::OciErrorEnvelope;

#[derive(Debug, Error)]
pub enum OciError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("denied")]
    Denied,
    #[error("name unknown")]
    NameUnknown,
    #[error("manifest unknown")]
    ManifestUnknown,
    #[error("blob unknown")]
    BlobUnknown,
    #[error("too many requests")]
    TooManyRequests { retry_after_secs: Option<u64> },
    #[error("upstream unavailable")]
    Upstream,
    #[error("unsupported")]
    Unsupported,
    #[error("internal")]
    Internal,
}

impl OciError {
    fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Denied => "DENIED",
            Self::NameUnknown => "NAME_UNKNOWN",
            Self::ManifestUnknown => "MANIFEST_UNKNOWN",
            Self::BlobUnknown => "BLOB_UNKNOWN",
            Self::TooManyRequests { .. } => "TOOMANYREQUESTS",
            Self::Upstream => "UNKNOWN",
            Self::Unsupported => "UNSUPPORTED",
            Self::Internal => "UNKNOWN",
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::Unauthorized => "authentication required",
            Self::Denied => "access denied",
            Self::NameUnknown => "repository name not known",
            Self::ManifestUnknown => "manifest not known",
            Self::BlobUnknown => "blob not known",
            Self::TooManyRequests { .. } => "too many requests",
            Self::Upstream => "upstream error",
            Self::Unsupported => "unsupported operation",
            Self::Internal => "internal error",
        }
    }
}

impl ResponseError for OciError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Denied => StatusCode::FORBIDDEN,
            Self::NameUnknown | Self::ManifestUnknown | Self::BlobUnknown => StatusCode::NOT_FOUND,
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Upstream => StatusCode::BAD_GATEWAY,
            Self::Unsupported => StatusCode::METHOD_NOT_ALLOWED,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let envelope = OciErrorEnvelope::single(self.code(), self.message());
        let mut builder = HttpResponse::build(self.status_code());
        builder.content_type("application/json");
        if let Self::TooManyRequests { retry_after_secs: Some(secs) } = self {
            builder.insert_header(("Retry-After", secs.to_string()));
        }
        builder.json(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_match_spec() {
        assert_eq!(OciError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(OciError::Denied.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(OciError::ManifestUnknown.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(OciError::BlobUnknown.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            OciError::TooManyRequests { retry_after_secs: None }.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(OciError::Upstream.status_code(), StatusCode::BAD_GATEWAY);
        assert_eq!(OciError::Unsupported.status_code(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn error_body_uses_oci_envelope() {
        let resp = OciError::ManifestUnknown.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn retry_after_set_on_daily_cap() {
        let resp = OciError::TooManyRequests { retry_after_secs: Some(3600) }.error_response();
        let val = resp.headers().get("Retry-After").unwrap();
        assert_eq!(val, "3600");
    }
}
```

- [ ] **Step 2: Export from errors module**

Add to `api/src/errors/mod.rs` (or `errors.rs`):

```rust
pub mod oci;
pub use oci::OciError;
```

- [ ] **Step 3: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test oci::tests --lib -- --nocapture`
(Yes, module name collides with models::oci — scope by path if needed: `cargo test errors::oci -- --nocapture`.)
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/errors.rs api/src/errors/
git commit -m "feat(oci-registry): add OciError with OCI error envelope"
```

---

## Task 10: `ForgejoRegistryClient`

**Files:**
- Create: `api/src/services/forgejo_registry.rs`
- Modify: `api/src/services/mod.rs`

- [ ] **Step 1: Write the client**

```rust
//! HTTP client for Forgejo's OCI distribution endpoints (/v2/*).
//!
//! Validates that every upstream URL is under the configured `base_url`
//! before forwarding the Forgejo API token, matching the safety rule in
//! `services::forgejo`.

use bytes::Bytes;
use futures_util::Stream;
use reqwest::{Client, Response};
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("not found")]
    NotFound,
    #[error("upstream status {0}")]
    Upstream(u16),
    #[error("invalid upstream url")]
    InvalidUrl,
    #[error("url parse: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// A manifest response from upstream: raw bytes + media type + digest.
pub struct ManifestResponse {
    pub bytes: Bytes,
    pub media_type: String,
    pub digest: String,
}

/// A streamed blob response: headers + body stream.
pub struct BlobStream {
    pub content_length: Option<u64>,
    pub media_type: Option<String>,
    pub digest: Option<String>,
    pub body: Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin>,
}

#[derive(Clone)]
pub struct ForgejoRegistryClient {
    http: Client,
    base_url: String,
    token: String,
}

impl ForgejoRegistryClient {
    pub fn new(base_url: String, token: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("reqwest client builds");
        Self { http, base_url, token }
    }

    fn validate_host(&self, url: &Url) -> Result<(), RegistryError> {
        let base = Url::parse(&self.base_url)?;
        if url.scheme() != base.scheme()
            || url.host_str() != base.host_str()
            || url.port_or_known_default() != base.port_or_known_default()
        {
            return Err(RegistryError::InvalidUrl);
        }
        Ok(())
    }

    /// GET /v2/<owner>/<name>/manifests/<reference>
    pub async fn get_manifest(
        &self,
        owner: &str,
        name: &str,
        reference: &str,
        accept: &str,
    ) -> Result<ManifestResponse, RegistryError> {
        let url = format!(
            "{}/v2/{}/{}/manifests/{}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(owner),
            urlencoding::encode(name),
            urlencoding::encode(reference),
        );
        let parsed = Url::parse(&url)?;
        self.validate_host(&parsed)?;

        let resp = self.http.get(parsed)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", accept)
            .send()
            .await?;

        self.map_status(&resp)?;
        let media_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/vnd.oci.image.manifest.v1+json")
            .to_string();
        let digest = resp
            .headers()
            .get("docker-content-digest")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let bytes = resp.bytes().await?;
        Ok(ManifestResponse { bytes, media_type, digest })
    }

    /// GET /v2/<owner>/<name>/blobs/<digest>
    pub async fn get_blob(
        &self,
        owner: &str,
        name: &str,
        digest: &str,
    ) -> Result<BlobStream, RegistryError> {
        let url = format!(
            "{}/v2/{}/{}/blobs/{}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(owner),
            urlencoding::encode(name),
            urlencoding::encode(digest),
        );
        let parsed = Url::parse(&url)?;
        self.validate_host(&parsed)?;

        let resp = self.http.get(parsed)
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await?;

        self.map_status(&resp)?;
        let content_length = resp.content_length();
        let media_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let header_digest = resp
            .headers()
            .get("docker-content-digest")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let stream = resp.bytes_stream();
        Ok(BlobStream {
            content_length,
            media_type,
            digest: header_digest,
            body: Box::new(stream),
        })
    }

    fn map_status(&self, resp: &Response) -> Result<(), RegistryError> {
        match resp.status().as_u16() {
            200 => Ok(()),
            404 => Err(RegistryError::NotFound),
            code if (500..600).contains(&code) => Err(RegistryError::Upstream(code)),
            code => Err(RegistryError::Upstream(code)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[actix_rt::test]
    async fn rejects_upstream_url_outside_base() {
        // Construct client with one base; the internal validate_host should reject
        // a URL pointing at a different host even if the caller hands it in.
        let client = ForgejoRegistryClient::new(
            "https://git.example.com".into(),
            "tok".into(),
        );
        let bad = Url::parse("https://evil.example.com/v2/a/b/blobs/sha256:x").unwrap();
        assert!(matches!(client.validate_host(&bad), Err(RegistryError::InvalidUrl)));

        let good = Url::parse("https://git.example.com/v2/a/b/blobs/sha256:x").unwrap();
        assert!(client.validate_host(&good).is_ok());
    }

    #[actix_rt::test]
    async fn get_manifest_returns_headers_and_body() {
        let server = MockServer::start().await;
        let body = br#"{"mediaType":"application/vnd.oci.image.manifest.v1+json","layers":[]}"#;
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/manifests/.+"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(body.to_vec())
                    .insert_header("Content-Type", "application/vnd.oci.image.manifest.v1+json")
                    .insert_header("Docker-Content-Digest", "sha256:abc"),
            )
            .mount(&server)
            .await;

        let client = ForgejoRegistryClient::new(server.uri(), "tok".into());
        let mr = client.get_manifest("a", "b", "v1", "application/vnd.oci.image.manifest.v1+json").await.unwrap();
        assert_eq!(mr.media_type, "application/vnd.oci.image.manifest.v1+json");
        assert_eq!(mr.digest, "sha256:abc");
        assert_eq!(mr.bytes, Bytes::from(&body[..]));
    }

    #[actix_rt::test]
    async fn upstream_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/manifests/.+"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = ForgejoRegistryClient::new(server.uri(), "tok".into());
        let err = client.get_manifest("a", "b", "v1", "application/json").await.unwrap_err();
        assert!(matches!(err, RegistryError::NotFound));
    }
}
```

- [ ] **Step 2: Export from `services/mod.rs`**

```rust
pub mod forgejo_registry;
pub use forgejo_registry::{ForgejoRegistryClient, RegistryError};
```

- [ ] **Step 3: Ensure `url` is available in `Cargo.toml`**

Run: `docker compose -f compose.dev.yml exec api cargo add url`
Expected: `url` added as a dependency (it may already be present transitively — if the add says it's already in the tree, skip).

- [ ] **Step 4: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test forgejo_registry --lib -- --nocapture`
Expected: all three tests PASS.

- [ ] **Step 5: Commit**

```bash
git add api/src/services/forgejo_registry.rs api/src/services/mod.rs api/Cargo.toml api/Cargo.lock
git commit -m "feat(oci-registry): add ForgejoRegistryClient with host validation"
```

---

## Task 11: `oci_blob_cache` repository

**Files:**
- Create: `api/src/repositories/oci_blob_cache.rs`
- Modify: `api/src/repositories/mod.rs`

- [ ] **Step 1: Write the repository**

```rust
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
        let (total,): (Option<i64>,) =
            sqlx::query_as("SELECT SUM(size_bytes) FROM oci_blob_cache")
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
    pub async fn delete_except(
        pool: &PgPool,
        keep: &[String],
    ) -> Result<Vec<String>, AppError> {
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
    use super::*;

    #[sqlx::test]
    async fn upsert_inserts_then_touches(pool: PgPool) {
        OciBlobCacheRepository::upsert(&pool, &NewCachedBlob {
            content_digest: "sha256:abc".into(),
            size_bytes: 100,
            media_type: Some("application/octet-stream".into()),
        }).await.unwrap();

        let first = OciBlobCacheRepository::find(&pool, "sha256:abc").await.unwrap().unwrap();
        let first_access = first.last_accessed_at;

        // Second upsert updates last_accessed_at.
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        OciBlobCacheRepository::upsert(&pool, &NewCachedBlob {
            content_digest: "sha256:abc".into(),
            size_bytes: 100,
            media_type: None,
        }).await.unwrap();

        let second = OciBlobCacheRepository::find(&pool, "sha256:abc").await.unwrap().unwrap();
        assert!(second.last_accessed_at > first_access);
        // media_type preserved via COALESCE.
        assert_eq!(second.media_type.as_deref(), Some("application/octet-stream"));
    }

    #[sqlx::test]
    async fn oldest_orders_by_last_accessed(pool: PgPool) {
        OciBlobCacheRepository::upsert(&pool, &NewCachedBlob {
            content_digest: "sha256:a".into(), size_bytes: 1, media_type: None,
        }).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        OciBlobCacheRepository::upsert(&pool, &NewCachedBlob {
            content_digest: "sha256:b".into(), size_bytes: 1, media_type: None,
        }).await.unwrap();

        let rows = OciBlobCacheRepository::oldest(&pool, 10).await.unwrap();
        assert_eq!(rows[0].content_digest, "sha256:a");
        assert_eq!(rows[1].content_digest, "sha256:b");
    }

    #[sqlx::test]
    async fn delete_except_removes_unlisted(pool: PgPool) {
        for d in ["sha256:a", "sha256:b", "sha256:c"] {
            OciBlobCacheRepository::upsert(&pool, &NewCachedBlob {
                content_digest: d.into(), size_bytes: 1, media_type: None,
            }).await.unwrap();
        }

        let deleted = OciBlobCacheRepository::delete_except(
            &pool, &["sha256:a".into(), "sha256:b".into()],
        ).await.unwrap();
        assert_eq!(deleted, vec!["sha256:c"]);

        assert!(OciBlobCacheRepository::find(&pool, "sha256:a").await.unwrap().is_some());
        assert!(OciBlobCacheRepository::find(&pool, "sha256:c").await.unwrap().is_none());
    }
}
```

- [ ] **Step 2: Export from `repositories/mod.rs`**

```rust
pub mod oci_blob_cache;
pub use oci_blob_cache::OciBlobCacheRepository;
```

- [ ] **Step 3: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test oci_blob_cache --lib -- --nocapture`
Expected: three tests PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/repositories/oci_blob_cache.rs api/src/repositories/mod.rs
git commit -m "feat(oci-registry): OciBlobCacheRepository"
```

---

## Task 12: `oci_pull_daily_counts` repository

**Files:**
- Create: `api/src/repositories/oci_pull_daily_counts.rs`
- Modify: `api/src/repositories/mod.rs`

- [ ] **Step 1: Write the repository**

```rust
//! DB access for the `oci_pull_daily_counts` table.

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;

pub struct OciPullDailyCountRepository;

impl OciPullDailyCountRepository {
    /// Atomically increment today's count for a user. Returns the new count.
    pub async fn increment(
        pool: &PgPool,
        user_id: Uuid,
        day_utc: NaiveDate,
    ) -> Result<i32, AppError> {
        let (count,): (i32,) = sqlx::query_as(
            "INSERT INTO oci_pull_daily_counts (user_id, day_utc, count)
             VALUES ($1, $2, 1)
             ON CONFLICT (user_id, day_utc) DO UPDATE
                 SET count = oci_pull_daily_counts.count + 1
             RETURNING count",
        )
        .bind(user_id)
        .bind(day_utc)
        .fetch_one(pool)
        .await?;
        Ok(count)
    }

    /// Decrement today's count by 1 (best-effort rollback). Never goes below 0.
    pub async fn decrement(
        pool: &PgPool,
        user_id: Uuid,
        day_utc: NaiveDate,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE oci_pull_daily_counts SET count = GREATEST(count - 1, 0)
             WHERE user_id = $1 AND day_utc = $2",
        )
        .bind(user_id)
        .bind(day_utc)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn current(
        pool: &PgPool,
        user_id: Uuid,
        day_utc: NaiveDate,
    ) -> Result<i32, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT count FROM oci_pull_daily_counts WHERE user_id = $1 AND day_utc = $2",
        )
        .bind(user_id)
        .bind(day_utc)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|(c,)| c).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[sqlx::test]
    async fn increment_creates_and_bumps(pool: PgPool) {
        let user = Uuid::new_v4();
        // Insert a user row first (FK).
        sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, 'x', 'subscriber')")
            .bind(user)
            .bind(format!("{}@test.com", user))
            .execute(&pool).await.unwrap();

        let today = Utc::now().date_naive();
        assert_eq!(OciPullDailyCountRepository::increment(&pool, user, today).await.unwrap(), 1);
        assert_eq!(OciPullDailyCountRepository::increment(&pool, user, today).await.unwrap(), 2);
        assert_eq!(OciPullDailyCountRepository::current(&pool, user, today).await.unwrap(), 2);

        OciPullDailyCountRepository::decrement(&pool, user, today).await.unwrap();
        assert_eq!(OciPullDailyCountRepository::current(&pool, user, today).await.unwrap(), 1);
    }
}
```

- [ ] **Step 2: Export from `repositories/mod.rs`**

```rust
pub mod oci_pull_daily_counts;
pub use oci_pull_daily_counts::OciPullDailyCountRepository;
```

- [ ] **Step 3: Run test**

Run: `docker compose -f compose.dev.yml exec api cargo test oci_pull_daily_counts --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/repositories/oci_pull_daily_counts.rs api/src/repositories/mod.rs
git commit -m "feat(oci-registry): OciPullDailyCountRepository"
```

---

## Task 13: `ManifestCache` service

**Files:**
- Create: `api/src/services/manifest_cache.rs`
- Modify: `api/src/services/mod.rs`

- [ ] **Step 1: Write the service**

```rust
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
        // moka doesn't support range invalidation; iterate.
        self.cache.invalidate_entries_if(move |(a, _), _| *a == app_id).ok();
        self.cache.run_pending_tasks().await;
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
```

- [ ] **Step 2: Export from `services/mod.rs`**

```rust
pub mod manifest_cache;
pub use manifest_cache::ManifestCache;
```

- [ ] **Step 3: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test manifest_cache --lib -- --nocapture`
Expected: two tests PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/services/manifest_cache.rs api/src/services/mod.rs
git commit -m "feat(oci-registry): ManifestCache with TTL + per-app invalidation"
```

---

## Task 14: `BlobCache` service (on-disk, single-flight, digest-verified)

**Files:**
- Create: `api/src/services/blob_cache.rs`
- Modify: `api/src/services/mod.rs`

- [ ] **Step 1: Write the service**

```rust
//! On-disk blob cache for OCI layers.
//!
//! Keyed by content digest. Single-flight fetches via Arc<OnceCell>.
//! Bytes are streamed from upstream into a `.partial` file while being
//! hashed; on mismatch the partial is deleted and no DB row is inserted.
//! Successful writes are atomically renamed to the final name, then an
//! async LRU eviction task runs if we're over the byte cap.

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
pub struct BlobHandle {
    pub digest: String,
    pub size_bytes: i64,
    pub media_type: Option<String>,
    pub path: PathBuf,
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

    pub async fn ensure_dir(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.cache_dir).await
    }

    fn final_path(&self, digest: &str) -> PathBuf {
        // digest is "sha256:<hex>"; flatten colon to avoid filesystem oddities.
        self.cache_dir.join(digest.replace(':', "_"))
    }

    fn partial_path(&self, digest: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.partial", digest.replace(':', "_")))
    }

    /// Fetch blob (cache on miss), returning a BlobHandle with path on disk.
    pub async fn get_or_fetch(
        &self,
        owner: &str,
        name: &str,
        digest: &str,
    ) -> Result<BlobHandle, AppError> {
        // 1. Fast-path: already on disk and in DB.
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

        // 2. Single-flight: one fetcher per digest.
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

        result.map_err(|s| AppError::internal(&format!("blob cache fetch: {s}")))?;

        let row = OciBlobCacheRepository::find(&self.pool, digest).await?
            .ok_or_else(|| AppError::internal("blob cache inconsistent: row missing after fetch"))?;
        Ok(BlobHandle {
            digest: row.content_digest,
            size_bytes: row.size_bytes,
            media_type: row.media_type,
            path: self.final_path(digest),
        })
    }

    async fn fetch_and_store(
        &self,
        owner: &str,
        name: &str,
        digest: &str,
    ) -> Result<(), AppError> {
        self.ensure_dir().await.map_err(|e| AppError::internal(&format!("mkdir: {e}")))?;
        let partial = self.partial_path(digest);
        let final_ = self.final_path(digest);

        let mut upstream = self.client.get_blob(owner, name, digest).await
            .map_err(map_registry_err)?;

        let mut file = fs::File::create(&partial).await
            .map_err(|e| AppError::internal(&format!("partial create: {e}")))?;
        let mut hasher = Sha256::new();
        let mut total: u64 = 0;

        while let Some(chunk) = upstream.body.next().await {
            let chunk: Bytes = chunk.map_err(|e| AppError::internal(&format!("upstream stream: {e}")))?;
            hasher.update(&chunk);
            file.write_all(&chunk).await
                .map_err(|e| AppError::internal(&format!("partial write: {e}")))?;
            total += chunk.len() as u64;
        }
        file.flush().await.ok();
        file.sync_all().await.ok();
        drop(file);

        let computed = format!("sha256:{}", hex::encode(hasher.finalize()));
        if computed != digest {
            fs::remove_file(&partial).await.ok();
            return Err(AppError::internal(&format!(
                "digest mismatch: upstream={computed}, expected={digest}"
            )));
        }

        fs::rename(&partial, &final_).await
            .map_err(|e| AppError::internal(&format!("rename: {e}")))?;

        OciBlobCacheRepository::upsert(&self.pool, &NewCachedBlob {
            content_digest: digest.to_string(),
            size_bytes: total as i64,
            media_type: upstream.media_type,
        }).await?;

        // Async eviction — don't block the fetch path.
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
            if total <= self.max_bytes { break; }
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
        _ => AppError::bad_gateway("upstream registry"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Helper to hex-hash a byte slice.
    fn digest_of(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        format!("sha256:{}", hex::encode(h.finalize()))
    }

    #[sqlx::test]
    async fn fetches_and_stores_blob(pool: PgPool) {
        let server = MockServer::start().await;
        let body = b"hello-oci".to_vec();
        let digest = digest_of(&body);

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
        let cache = BlobCache::new(client, tmp.path().to_str().unwrap(), 1_000_000, pool.clone());

        let handle = cache.get_or_fetch("a", "b", &digest).await.unwrap();
        assert_eq!(handle.size_bytes, body.len() as i64);
        assert!(handle.path.exists());
        let on_disk = fs::read(&handle.path).await.unwrap();
        assert_eq!(on_disk, body);
    }

    #[sqlx::test]
    async fn digest_mismatch_deletes_partial_and_no_row(pool: PgPool) {
        let server = MockServer::start().await;
        let body = b"corrupt".to_vec();
        let wrong_digest = digest_of(b"something-else");

        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/blobs/.+"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
            .mount(&server)
            .await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let tmp = tempfile::tempdir().unwrap();
        let cache = BlobCache::new(client, tmp.path().to_str().unwrap(), 1_000_000, pool.clone());

        let err = cache.get_or_fetch("a", "b", &wrong_digest).await.unwrap_err();
        assert!(err.to_string().contains("digest mismatch") || err.to_string().contains("blob cache fetch"));
        assert!(OciBlobCacheRepository::find(&pool, &wrong_digest).await.unwrap().is_none());
        // Partial and final both absent.
        let partial = tmp.path().join(format!("{}.partial", wrong_digest.replace(':', "_")));
        let final_ = tmp.path().join(wrong_digest.replace(':', "_"));
        assert!(!partial.exists());
        assert!(!final_.exists());
    }

    #[sqlx::test]
    async fn single_flight_merges_concurrent_fetches(pool: PgPool) {
        let server = MockServer::start().await;
        let body = b"dedup".to_vec();
        let digest = digest_of(&body);

        Mock::given(method("GET"))
            .and(path_regex("/v2/.+/blobs/.+"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
            .expect(1)
            .mount(&server)
            .await;

        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let tmp = tempfile::tempdir().unwrap();
        let cache = BlobCache::new(client, tmp.path().to_str().unwrap(), 1_000_000, pool.clone());

        let (a, b) = tokio::join!(
            cache.get_or_fetch("a", "b", &digest),
            cache.get_or_fetch("a", "b", &digest),
        );
        a.unwrap();
        b.unwrap();
        // Wiremock .expect(1) enforces the single upstream call.
    }
}
```

- [ ] **Step 2: Export from `services/mod.rs`**

```rust
pub mod blob_cache;
pub use blob_cache::{BlobCache, BlobHandle};
```

- [ ] **Step 3: Add `AppError::bad_gateway` if missing**

Grep for it: `grep -n "bad_gateway" api/src/errors*`. If missing, add to the `AppError` impl block:

```rust
pub fn bad_gateway(what: &str) -> Self {
    AppError::UpstreamUnavailable(what.to_string())
}
```

(If `UpstreamUnavailable` doesn't exist either, reuse the existing "upstream" variant used by the download proxy — check `handlers/download.rs` for the exact pattern it uses. Consistency with the download flow matters.)

- [ ] **Step 4: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test blob_cache --lib -- --nocapture`
Expected: three tests PASS.

- [ ] **Step 5: Commit**

```bash
git add api/src/services/blob_cache.rs api/src/services/mod.rs api/src/errors*
git commit -m "feat(oci-registry): BlobCache with single-flight, digest verification, LRU eviction"
```

---

## Task 15: `OciTokenService`

**Files:**
- Create: `api/src/services/oci_token.rs`
- Modify: `api/src/services/mod.rs`

- [ ] **Step 1: Write the service**

```rust
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
        // Hand-craft a token with aud="api" using the same secret.
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
```

- [ ] **Step 2: Export from `services/mod.rs`**

```rust
pub mod oci_token;
pub use oci_token::{OciTokenService, RegistryTokenClaims, REGISTRY_AUDIENCE};
```

- [ ] **Step 3: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test oci_token --lib -- --nocapture`
Expected: three tests PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/services/oci_token.rs api/src/services/mod.rs
git commit -m "feat(oci-registry): OciTokenService for bearer issuance + verification"
```

---

## Task 16: `OciLimiter` service

**Files:**
- Create: `api/src/services/oci_limiter.rs`
- Modify: `api/src/services/mod.rs`

- [ ] **Step 1: Write the service**

```rust
//! Per-user concurrency (manifest fetches only) + daily pull counter.
//!
//! Mirrors `DownloadLimiter`. In-process concurrency map is single-
//! instance only — Postgres-backed replacement tracked as follow-up.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::OciPullDailyCountRepository;

#[derive(Debug, PartialEq)]
pub enum OciLimitDenial {
    Concurrency,
    DailyCap { reset_in_secs: i64 },
}

#[derive(Clone)]
pub struct OciLimiter {
    concurrency_per_user: u32,
    daily_limit: u32,
    inflight: Arc<Mutex<HashMap<Uuid, u32>>>,
}

/// RAII guard that releases a concurrency slot on drop.
pub struct OciPullGuard {
    user_id: Uuid,
    inflight: Arc<Mutex<HashMap<Uuid, u32>>>,
}

impl Drop for OciPullGuard {
    fn drop(&mut self) {
        let mut m = self.inflight.lock().unwrap();
        if let Some(n) = m.get_mut(&self.user_id) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                m.remove(&self.user_id);
            }
        }
    }
}

impl OciLimiter {
    pub fn new(concurrency_per_user: u32, daily_limit: u32) -> Self {
        Self {
            concurrency_per_user,
            daily_limit,
            inflight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Try to acquire a manifest-pull slot. Returns a guard on success.
    pub async fn acquire(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Result<OciPullGuard, OciLimitDenial>, AppError> {
        // 1. Concurrency.
        {
            let mut m = self.inflight.lock().unwrap();
            let entry = m.entry(user_id).or_insert(0);
            if *entry >= self.concurrency_per_user {
                return Ok(Err(OciLimitDenial::Concurrency));
            }
            *entry += 1;
        }

        // 2. Daily counter.
        let today = Utc::now().date_naive();
        let count = match OciPullDailyCountRepository::increment(pool, user_id, today).await {
            Ok(n) => n,
            Err(e) => {
                // Release concurrency slot.
                let mut m = self.inflight.lock().unwrap();
                if let Some(n) = m.get_mut(&user_id) {
                    *n = n.saturating_sub(1);
                    if *n == 0 { m.remove(&user_id); }
                }
                return Err(e);
            }
        };

        if (count as u32) > self.daily_limit {
            // Roll back counter + release slot.
            OciPullDailyCountRepository::decrement(pool, user_id, today).await.ok();
            let mut m = self.inflight.lock().unwrap();
            if let Some(n) = m.get_mut(&user_id) {
                *n = n.saturating_sub(1);
                if *n == 0 { m.remove(&user_id); }
            }
            let reset_in_secs = seconds_until_utc_midnight();
            return Ok(Err(OciLimitDenial::DailyCap { reset_in_secs }));
        }

        Ok(Ok(OciPullGuard {
            user_id,
            inflight: self.inflight.clone(),
        }))
    }
}

fn seconds_until_utc_midnight() -> i64 {
    let now = Utc::now();
    let tomorrow = (now + chrono::Duration::days(1)).date_naive();
    let midnight = tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc();
    (midnight - now).num_seconds().max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn guard_releases_slot_on_drop(pool: PgPool) {
        let user = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, 'x', 'subscriber')")
            .bind(user).bind(format!("{}@t.com", user)).execute(&pool).await.unwrap();

        let limiter = OciLimiter::new(1, 50);
        {
            let guard = limiter.acquire(&pool, user).await.unwrap().unwrap();
            // Another acquire should deny.
            match limiter.acquire(&pool, user).await.unwrap() {
                Err(OciLimitDenial::Concurrency) => {}
                other => panic!("expected Concurrency denial, got {other:?}"),
            }
            drop(guard);
        }
        // After drop, acquire succeeds again.
        let _ = limiter.acquire(&pool, user).await.unwrap().unwrap();
    }

    #[sqlx::test]
    async fn daily_cap_denies_over_limit(pool: PgPool) {
        let user = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, 'x', 'subscriber')")
            .bind(user).bind(format!("{}@t.com", user)).execute(&pool).await.unwrap();

        let limiter = OciLimiter::new(5, 2);
        let g1 = limiter.acquire(&pool, user).await.unwrap().unwrap();
        drop(g1);
        let g2 = limiter.acquire(&pool, user).await.unwrap().unwrap();
        drop(g2);

        match limiter.acquire(&pool, user).await.unwrap() {
            Err(OciLimitDenial::DailyCap { reset_in_secs }) => {
                assert!(reset_in_secs > 0);
                assert!(reset_in_secs <= 86_400);
            }
            other => panic!("expected DailyCap denial, got {other:?}"),
        }

        // Counter rolled back to 2 after the denial.
        let today = Utc::now().date_naive();
        let cur = OciPullDailyCountRepository::current(&pool, user, today).await.unwrap();
        assert_eq!(cur, 2);
    }
}
```

- [ ] **Step 2: Export from `services/mod.rs`**

```rust
pub mod oci_limiter;
pub use oci_limiter::{OciLimiter, OciLimitDenial, OciPullGuard};
```

- [ ] **Step 3: Run tests**

Run: `docker compose -f compose.dev.yml exec api cargo test oci_limiter --lib -- --nocapture`
Expected: two tests PASS.

- [ ] **Step 4: Commit**

```bash
git add api/src/services/oci_limiter.rs api/src/services/mod.rs
git commit -m "feat(oci-registry): OciLimiter with concurrency + daily caps"
```

---

## Task 17: `OciBearerUser` extractor

**Files:**
- Create: `api/src/middleware/oci_auth.rs`
- Modify: `api/src/middleware/mod.rs`

- [ ] **Step 1: Write the extractor**

```rust
//! Bearer-token extractor for the OCI registry server.
//!
//! - Validates the token (aud=registry, exp, iss) via `OciTokenService`.
//! - Re-loads the user + membership on every request.
//! - Does NOT enforce scope — handlers that take a `<slug>` are
//!   responsible for calling `claims.assert_scope(slug)`.

use actix_web::{dev::Payload, FromRequest, HttpRequest};
use chrono::Utc;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::errors::OciError;
use crate::repositories::{MembershipRepository, UserRepository};
use crate::services::oci_token::{OciTokenService, RegistryTokenClaims};

#[derive(Debug, Clone)]
pub struct OciBearerUser {
    pub claims: RegistryTokenClaims,
}

impl OciBearerUser {
    pub fn assert_scope(&self, slug: &str) -> Result<(), OciError> {
        let expected = format!("repository:{slug}:pull");
        if self.claims.scope == expected {
            Ok(())
        } else {
            Err(OciError::Denied)
        }
    }
}

impl FromRequest for OciBearerUser {
    type Error = OciError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let header = req.headers().get(actix_web::http::header::AUTHORIZATION).cloned();
        let token_svc = req.app_data::<Arc<OciTokenService>>().cloned();
        let pool = req.app_data::<actix_web::web::Data<PgPool>>().cloned();

        Box::pin(async move {
            let svc = token_svc.ok_or(OciError::Internal)?;
            let pool = pool.ok_or(OciError::Internal)?;

            let raw = header
                .and_then(|v| v.to_str().ok().map(str::to_string))
                .ok_or(OciError::Unauthorized)?;
            let token = raw.strip_prefix("Bearer ").ok_or(OciError::Unauthorized)?;
            let claims = svc.verify(token)?;

            // Re-check user + membership on every request.
            let user = UserRepository::find_by_id(pool.get_ref(), claims.sub)
                .await
                .map_err(|_| OciError::Internal)?
                .ok_or(OciError::Unauthorized)?;
            if !user.is_active {
                return Err(OciError::Unauthorized);
            }
            if user.role != "admin" {
                // Admins bypass membership check (matches rest of platform).
                let m = MembershipRepository::find_active_for_user(pool.get_ref(), user.id)
                    .await
                    .map_err(|_| OciError::Internal)?;
                let has_access = m.map(|row| {
                    row.status == "active"
                        && row.current_period_end.map_or(true, |end| end > Utc::now())
                }).unwrap_or(false);
                if !has_access && !user.lifetime_member {
                    return Err(OciError::Unauthorized);
                }
            }

            Ok(OciBearerUser { claims })
        })
    }
}
```

> The exact methods called on `UserRepository` / `MembershipRepository` must match the codebase — grep for the current names. If `MembershipRepository::find_active_for_user` doesn't exist, use whatever the rest of the auth flow uses (likely `find_by_user_id` + manual status check). The point is: re-check status, don't trust the JWT alone.

- [ ] **Step 2: Export from `middleware/mod.rs`**

```rust
pub mod oci_auth;
pub use oci_auth::OciBearerUser;
```

- [ ] **Step 3: Write tests**

Append to the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::JwtConfig;

    #[test]
    fn assert_scope_accepts_matching_slug() {
        let svc = OciTokenService::new(
            &JwtConfig::from_secret("a-very-long-secret-key-for-tests-12345", "a8n"),
            900,
        );
        let _ = svc; // silence unused warning
        let user = OciBearerUser {
            claims: RegistryTokenClaims {
                sub: uuid::Uuid::new_v4(),
                aud: "registry".into(),
                scope: "repository:my-app:pull".into(),
                iat: 0, exp: i64::MAX, iss: "a8n".into(),
            },
        };
        assert!(user.assert_scope("my-app").is_ok());
        assert!(matches!(user.assert_scope("other"), Err(OciError::Denied)));
    }
}
```

(The full extractor is hard to unit-test without a running Actix app — the integration test in Task 23 covers it end-to-end.)

- [ ] **Step 4: Run the unit test**

Run: `docker compose -f compose.dev.yml exec api cargo test oci_auth --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add api/src/middleware/oci_auth.rs api/src/middleware/mod.rs
git commit -m "feat(oci-registry): OciBearerUser extractor"
```

---

## Task 18: `/auth/token` handler

**Files:**
- Create: `api/src/handlers/oci_auth.rs`
- Modify: `api/src/handlers/mod.rs`

- [ ] **Step 1: Write the handler**

```rust
//! Registry bearer-token handler (`GET /auth/token`).
//!
//! Docker clients call this with basic auth (email:password) after
//! getting a 401+WWW-Authenticate from `/v2/`.

use actix_web::{web, HttpRequest, HttpResponse};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use crate::errors::OciError;
use crate::middleware::extract_client_ip;
use crate::models::{AuditAction, CreateAuditLog};
use crate::repositories::{
    ApplicationRepository, AuditLogRepository, MembershipRepository, UserRepository,
};
use crate::services::{AuthService, OciTokenService};

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

/// GET /auth/token
pub async fn issue_token(
    req: HttpRequest,
    query: web::Query<TokenQuery>,
    pool: web::Data<PgPool>,
    auth: web::Data<Arc<AuthService>>,
    token_svc: web::Data<Arc<OciTokenService>>,
) -> Result<HttpResponse, OciError> {
    let ip = extract_client_ip(&req);
    let (email, password) = parse_basic_auth(&req).ok_or(OciError::Unauthorized)?;

    let user = UserRepository::find_by_email(pool.get_ref(), &email)
        .await
        .map_err(|_| OciError::Internal)?
        .ok_or(OciError::Unauthorized)?;

    if !user.is_active {
        audit_failed(&pool, &req, &email, ip, "inactive_user").await;
        return Err(OciError::Unauthorized);
    }

    let password_ok = auth.verify_password_for(&user, &password).await
        .map_err(|_| OciError::Internal)?;
    if !password_ok {
        audit_failed(&pool, &req, &email, ip, "bad_password").await;
        return Err(OciError::Unauthorized);
    }

    // Membership check (admins bypass).
    if user.role != "admin" && !user.lifetime_member {
        let m = MembershipRepository::find_active_for_user(pool.get_ref(), user.id)
            .await
            .map_err(|_| OciError::Internal)?;
        let active = m.map(|row| {
            row.status == "active"
                && row.current_period_end.map_or(true, |end| end > Utc::now())
        }).unwrap_or(false);
        if !active {
            audit_failed(&pool, &req, &email, ip, "no_active_membership").await;
            return Err(OciError::Unauthorized);
        }
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
        .with_metadata(serde_json::json!({"scope": scope_str}));
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
    req: &HttpRequest,
    email: &str,
    ip: Option<ipnetwork::IpNetwork>,
    reason: &str,
) {
    let _ = req; // reserved for future request-id capture
    let log = CreateAuditLog::new(AuditAction::OciLoginFailed)
        .with_ip(ip)
        .with_metadata(serde_json::json!({"email": email, "reason": reason}));
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
            .insert_header(("Authorization", format!("Basic {}", STANDARD.encode("me@example.com:hunter2"))))
            .to_http_request();
        assert_eq!(
            parse_basic_auth(&req),
            Some(("me@example.com".into(), "hunter2".into()))
        );
    }
}
```

> `AuthService::verify_password_for` may not exist under that exact name — adapt to whatever the codebase exposes (e.g., `password_service.verify(&user.password_hash, &password)`). The point is: never re-hash the password here; delegate to existing auth primitives.

- [ ] **Step 2: Export from `handlers/mod.rs`**

```rust
pub mod oci_auth;
```

- [ ] **Step 3: Ensure `base64` crate is present**

Run: `docker compose -f compose.dev.yml exec api cargo add base64` (skip if already a dep).

- [ ] **Step 4: Run unit tests**

Run: `docker compose -f compose.dev.yml exec api cargo test oci_auth --lib -- --nocapture`
Expected: both unit tests PASS (integration test comes in Task 23).

- [ ] **Step 5: Commit**

```bash
git add api/src/handlers/oci_auth.rs api/src/handlers/mod.rs api/Cargo.toml api/Cargo.lock
git commit -m "feat(oci-registry): /auth/token handler with basic-auth + membership check"
```

---

## Task 19: `/v2/*` handlers — version probe, manifest, blob, push-catchall

**Files:**
- Create: `api/src/handlers/oci_registry.rs`
- Modify: `api/src/handlers/mod.rs`

- [ ] **Step 1: Write the handlers**

```rust
//! /v2/* handlers for the OCI registry server.

use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::{web, HttpRequest, HttpResponse};
use bytes::Bytes;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio_util::codec::{BytesCodec, FramedRead};
use futures_util::StreamExt;

use crate::config::OciConfig;
use crate::errors::OciError;
use crate::middleware::{extract_client_ip, OciBearerUser};
use crate::models::oci::{CachedManifest, ParsedManifest};
use crate::models::{AuditAction, CreateAuditLog};
use crate::repositories::{ApplicationRepository, AuditLogRepository};
use crate::services::{
    BlobCache, ForgejoRegistryClient, ManifestCache, OciLimiter, OciLimitDenial,
};

fn www_authenticate(cfg: &OciConfig) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "Bearer realm=\"https://{service}/auth/token\",service=\"{service}\"",
        service = cfg.service
    )).unwrap()
}

/// GET /v2/
pub async fn version_probe(
    user: Option<OciBearerUser>,
    cfg: web::Data<OciConfig>,
) -> Result<HttpResponse, OciError> {
    match user {
        Some(_) => Ok(HttpResponse::Ok()
            .append_header(("Docker-Distribution-API-Version", "registry/2.0"))
            .finish()),
        None => Err(OciError::Unauthorized),
    }
}

/// GET /v2/{slug}/manifests/{reference}
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
    user.assert_scope(&slug)?;

    let client = client.as_ref().as_ref().ok_or(OciError::NameUnknown)?.clone();
    let cache = manifest_cache.as_ref().as_ref().ok_or(OciError::Internal)?.clone();

    let app = ApplicationRepository::find_active_by_slug(pool.get_ref(), &slug)
        .await.map_err(|_| OciError::Internal)?
        .ok_or(OciError::NameUnknown)?;
    if !app.is_pullable() {
        return Err(OciError::NameUnknown);
    }
    let pinned = app.pinned_image_tag.clone().unwrap();
    // Reference must be either the pinned tag or a digest reachable from it.
    // We enforce tag-match here; digests (sha256:...) are allowed for
    // multi-arch child manifests — they're validated against the pinned
    // tag's chain on-the-fly.
    let is_digest = reference.starts_with("sha256:");
    if !is_digest && reference != pinned {
        return Err(OciError::ManifestUnknown);
    }

    // Rate limit.
    let guard = match limiter.acquire(pool.get_ref(), user.claims.sub).await.map_err(|_| OciError::Internal)? {
        Ok(g) => g,
        Err(OciLimitDenial::Concurrency) => {
            audit_denied(&pool, &req, &user, &app.id, "concurrency", None).await;
            return Err(OciError::TooManyRequests { retry_after_secs: None });
        }
        Err(OciLimitDenial::DailyCap { reset_in_secs }) => {
            audit_denied(&pool, &req, &user, &app.id, "daily_cap", Some(reset_in_secs as u64)).await;
            return Err(OciError::TooManyRequests { retry_after_secs: Some(reset_in_secs as u64) });
        }
    };

    // Audit request.
    audit_requested(&pool, &req, &user, &app.id, &reference).await;

    // Cache hit?
    let accept = req.headers()
        .get(actix_web::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json")
        .to_string();

    let manifest = if let Some(hit) = cache.get(app.id, &reference).await {
        hit
    } else {
        let owner = app.oci_image_owner.as_deref().unwrap();
        let name  = app.oci_image_name.as_deref().unwrap();
        let mr = client.get_manifest(owner, name, &reference, &accept).await
            .map_err(map_reg_err)?;
        let digest = if mr.digest.is_empty() {
            format!("sha256:{}", hex::encode(sha2::Sha256::digest(&mr.bytes)))
        } else {
            mr.digest
        };
        cache.insert(app.id, &reference, CachedManifest {
            bytes: mr.bytes,
            media_type: mr.media_type,
            digest,
        }).await
    };

    audit_completed(&pool, &req, &user, &app.id, &reference, &manifest.digest).await;
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

/// GET /v2/{slug}/blobs/{digest}
pub async fn get_blob(
    req: HttpRequest,
    user: OciBearerUser,
    path: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    blob_cache: web::Data<Option<Arc<BlobCache>>>,
) -> Result<HttpResponse, OciError> {
    let (slug, digest) = path.into_inner();
    user.assert_scope(&slug)?;
    let blob_cache = blob_cache.as_ref().as_ref().ok_or(OciError::Internal)?.clone();

    let app = ApplicationRepository::find_active_by_slug(pool.get_ref(), &slug)
        .await.map_err(|_| OciError::Internal)?
        .ok_or(OciError::NameUnknown)?;
    if !app.is_pullable() {
        return Err(OciError::NameUnknown);
    }
    let owner = app.oci_image_owner.as_deref().unwrap();
    let name = app.oci_image_name.as_deref().unwrap();

    let handle = blob_cache.get_or_fetch(owner, name, &digest).await
        .map_err(|_| OciError::Upstream)?;

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

    let file = tokio::fs::File::open(&handle.path).await.map_err(|_| OciError::Internal)?;
    let stream = FramedRead::new(file, BytesCodec::new())
        .map(|r| r.map(|b| b.freeze()).map_err(|_| actix_web::error::ErrorInternalServerError("io")));
    Ok(resp.streaming(stream))
}

/// Catchall for push verbs on /v2/*.
pub async fn push_not_supported() -> Result<HttpResponse, OciError> {
    Err(OciError::Unsupported)
}

fn map_reg_err(e: crate::services::forgejo_registry::RegistryError) -> OciError {
    use crate::services::forgejo_registry::RegistryError;
    match e {
        RegistryError::NotFound => OciError::ManifestUnknown,
        _ => OciError::Upstream,
    }
}

async fn audit_requested(pool: &PgPool, req: &HttpRequest, user: &OciBearerUser, app_id: &uuid::Uuid, reference: &str) {
    let log = CreateAuditLog::new(AuditAction::OciPullRequested)
        .with_actor(user.claims.sub, "", "")
        .with_ip(extract_client_ip(req))
        .with_resource("application", *app_id)
        .with_metadata(serde_json::json!({"reference": reference}));
    let _ = AuditLogRepository::create(pool, log).await;
}

async fn audit_completed(pool: &PgPool, req: &HttpRequest, user: &OciBearerUser, app_id: &uuid::Uuid, reference: &str, digest: &str) {
    let log = CreateAuditLog::new(AuditAction::OciPullCompleted)
        .with_actor(user.claims.sub, "", "")
        .with_ip(extract_client_ip(req))
        .with_resource("application", *app_id)
        .with_metadata(serde_json::json!({"reference": reference, "digest": digest}));
    let _ = AuditLogRepository::create(pool, log).await;
}

async fn audit_denied(pool: &PgPool, req: &HttpRequest, user: &OciBearerUser, app_id: &uuid::Uuid, reason: &str, reset_in_secs: Option<u64>) {
    let log = CreateAuditLog::new(AuditAction::OciPullDeniedRateLimit)
        .with_actor(user.claims.sub, "", "")
        .with_ip(extract_client_ip(req))
        .with_resource("application", *app_id)
        .with_metadata(serde_json::json!({"reason": reason, "reset_in_secs": reset_in_secs}));
    let _ = AuditLogRepository::create(pool, log).await;
}
```

> Two places to watch:
> 1. `with_actor` requires email + role — look up from the user repo or change `CreateAuditLog` to accept a minimal actor (id only) for these member events. Cheapest: enrich by loading the user from the pool once per handler entry and stashing email/role on `OciBearerUser` in Task 17. Do that now — rename Task 17's struct to include `email` and `role` fields and fill them in the extractor.
> 2. The member-event audit rows won't match `is_admin_action = false` → correct; no change needed to `is_admin_action`.

Circle back and enrich `OciBearerUser { claims, email: String, role: String }` if you haven't — update the extractor to populate these from the `UserRepository::find_by_id` call it already makes.

- [ ] **Step 2: Export from `handlers/mod.rs`**

```rust
pub mod oci_registry;
```

- [ ] **Step 3: Commit (handlers compile; integration test lands in Task 23)**

Run: `docker compose -f compose.dev.yml exec api cargo build --lib`
Expected: builds cleanly.

```bash
git add api/src/handlers/oci_registry.rs api/src/handlers/mod.rs api/src/middleware/oci_auth.rs
git commit -m "feat(oci-registry): /v2/* handlers (version, manifest, blob, push-catchall)"
```

---

## Task 20: Routes — `api/src/routes/oci.rs` + factory

**Files:**
- Create: `api/src/routes/oci.rs`

- [ ] **Step 1: Write the route module**

```rust
//! Route configuration for the OCI registry server.

use actix_web::web;

use crate::handlers::{oci_auth, oci_registry};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/auth/token")
            .route(web::get().to(oci_auth::issue_token)),
    )
    .service(
        web::scope("/v2")
            .service(web::resource("").route(web::get().to(oci_registry::version_probe)))
            .service(web::resource("/").route(web::get().to(oci_registry::version_probe)))
            .service(
                web::resource("/{slug}/manifests/{reference}")
                    .route(web::get().to(oci_registry::get_manifest))
                    .route(web::head().to(oci_registry::get_manifest)),
            )
            .service(
                web::resource("/{slug}/blobs/{digest}")
                    .route(web::get().to(oci_registry::get_blob))
                    .route(web::head().to(oci_registry::get_blob)),
            )
            // Push-catchall: any non-GET/HEAD under /v2/* → 405.
            .service(
                web::resource("/{tail:.*}")
                    .route(web::post().to(oci_registry::push_not_supported))
                    .route(web::put().to(oci_registry::push_not_supported))
                    .route(web::patch().to(oci_registry::push_not_supported))
                    .route(web::delete().to(oci_registry::push_not_supported)),
            ),
    );
}
```

- [ ] **Step 2: Export from `routes/mod.rs`**

Add a new public module alongside the existing ones:

```rust
pub mod oci;
```

Do NOT call `oci::configure` from the main `routes::configure` — the OCI server is a separate `App`.

- [ ] **Step 3: Build**

Run: `docker compose -f compose.dev.yml exec api cargo build --lib`
Expected: builds cleanly.

- [ ] **Step 4: Commit**

```bash
git add api/src/routes/oci.rs api/src/routes/mod.rs
git commit -m "feat(oci-registry): registry route config"
```

---

## Task 21: Wire services + spawn second `HttpServer` in `main.rs`

**Files:**
- Modify: `api/src/main.rs`

- [ ] **Step 1: Initialize OCI services (next to the existing download block)**

Inside `main.rs` after the download-service initialization:

```rust
use a8n_api::services::{
    BlobCache, ManifestCache, OciLimiter, OciTokenService, ForgejoRegistryClient,
};

let forgejo_registry_client = config.download.forgejo_base_url.as_ref().and_then(|base| {
    config.download.forgejo_api_token.as_ref().map(|token| {
        Arc::new(ForgejoRegistryClient::new(base.clone(), token.clone()))
    })
});

let manifest_cache = forgejo_registry_client.as_ref().map(|_| {
    Arc::new(ManifestCache::new(config.oci.manifest_cache_ttl_secs))
});

let blob_cache = forgejo_registry_client.clone().map(|c| {
    Arc::new(BlobCache::new(
        c,
        &config.oci.blob_cache_dir,
        config.oci.blob_cache_max_bytes,
        pool.clone(),
    ))
});

if let Some(bc) = &blob_cache {
    if let Err(e) = bc.ensure_dir().await {
        tracing::warn!(error = %e, "failed to create oci blob cache dir");
    }
}

let oci_limiter = Arc::new(OciLimiter::new(
    config.oci.concurrent_manifests_per_user,
    config.oci.pulls_per_user_per_day,
));
let oci_token_service = Arc::new(OciTokenService::new(&jwt_config, config.oci.token_ttl_secs));

info!(
    enabled = config.oci.enabled,
    port = config.oci.port,
    "OCI registry service initialized"
);
```

(`jwt_config` may need to be kept in scope — currently `main.rs` builds `jwt_service` from it then drops it. Keep the `JwtConfig` around.)

- [ ] **Step 2: Register OCI services as `app_data` on the primary `App::new()` too**

The admin refresh endpoint (Task 25) lives on the primary API, and it needs `ManifestCache` + `BlobCache` for invalidation. Add to the existing `App::new()` chain:

```rust
.app_data(web::Data::new(manifest_cache.clone()))
.app_data(web::Data::new(blob_cache.clone()))
.app_data(web::Data::new(oci_limiter.clone()))
.app_data(web::Data::new(oci_token_service.clone()))
.app_data(web::Data::new(config_data.oci.clone()))
.app_data(web::Data::new(forgejo_registry_client.clone()))
```

- [ ] **Step 3: Build the second `HttpServer` and run both concurrently**

Replace the final `HttpServer::new(...).run().await?` block so that the primary + registry servers run via `tokio::try_join!`. Sketch:

```rust
let primary_future = HttpServer::new({
    // ... existing closure ...
})
.bind(&server_addr)?
.shutdown_timeout(30)
.run();

let oci_future = if config.oci.enabled {
    let oci_addr = format!("{}:{}", config.host, config.oci.port);
    let mc = manifest_cache.clone();
    let bc = blob_cache.clone();
    let ol = oci_limiter.clone();
    let ots = oci_token_service.clone();
    let cfg_oci = config.oci.clone();
    let frc = forgejo_registry_client.clone();
    let pool_oci = pool.clone();
    let auth_oci = auth_service.clone();
    let jwt_oci = jwt_service.clone();

    let srv = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(SecurityHeaders)
            .wrap(RequestIdMiddleware)
            .app_data(web::Data::new(pool_oci.clone()))
            .app_data(web::Data::new(auth_oci.clone()))
            .app_data(jwt_oci.clone())
            .app_data(ots.clone())
            .app_data(web::Data::new(ots.clone()))
            .app_data(web::Data::new(mc.clone()))
            .app_data(web::Data::new(bc.clone()))
            .app_data(web::Data::new(ol.clone()))
            .app_data(web::Data::new(cfg_oci.clone()))
            .app_data(web::Data::new(frc.clone()))
            .configure(a8n_api::routes::oci::configure)
    })
    .bind(&oci_addr)?
    .shutdown_timeout(30)
    .run();
    info!(address = %oci_addr, "Starting OCI registry server");
    Some(srv)
} else {
    info!("OCI registry server disabled (OCI_REGISTRY_ENABLED!=true)");
    None
};

// Await both.
match oci_future {
    Some(oci) => tokio::try_join!(primary_future, oci).map(|_| ())?,
    None => primary_future.await?,
}
```

> If `OciTokenService` is used by the extractor via `Arc<OciTokenService>` (not `web::Data`), register both forms: `.app_data(ots.clone())` for the `Arc<T>` form and `.app_data(web::Data::new(ots.clone()))` for the `web::Data` form. The extractor in Task 17 uses `Arc<OciTokenService>` via `app_data()`; match that.

- [ ] **Step 4: Build**

Run: `docker compose -f compose.dev.yml exec api cargo build --lib --bin a8n-api`
Expected: builds cleanly.

- [ ] **Step 5: Run the dev stack and probe `/v2/`**

```bash
just up
curl -i http://localhost:18081/v2/
```

Expected: `HTTP/1.1 401 Unauthorized` with `Content-Type: application/json` and a body matching `{"errors":[{"code":"UNAUTHORIZED",...}]}`.

> The `WWW-Authenticate` header is set in the next task — for now, 401 with the envelope is sufficient.

- [ ] **Step 6: Commit**

```bash
git add api/src/main.rs
git commit -m "feat(oci-registry): spawn oci registry HttpServer alongside primary"
```

---

## Task 22: Add `WWW-Authenticate` to OCI 401s

**Files:**
- Modify: `api/src/errors/oci.rs` OR `api/src/handlers/oci_registry.rs` (version_probe)

The cleanest place to emit `WWW-Authenticate` is in `OciError::error_response` for the `Unauthorized` variant, but that requires knowing the service string. Two options:

**A.** Thread `OciConfig` through the extractor and handlers so they build the header themselves on the 401 branch (overriding `error_response` via a wrapping `HttpResponse` before returning).

**B.** Put the header logic in a tiny middleware that runs on the OCI `App` only, rewriting 401 responses to include the header.

Option B is simpler. Implement as a per-response interceptor:

- [ ] **Step 1: Add a small wrapper middleware**

Create `api/src/middleware/oci_www_authenticate.rs`:

```rust
//! Adds `WWW-Authenticate: Bearer ...` to every 401 response on the OCI App.

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::sync::Arc;

use crate::config::OciConfig;

pub struct OciWwwAuthenticate {
    pub cfg: Arc<OciConfig>,
}

impl<S, B> Transform<S, ServiceRequest> for OciWwwAuthenticate
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = OciWwwAuthenticateMw<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(OciWwwAuthenticateMw { service, cfg: self.cfg.clone() })
    }
}

pub struct OciWwwAuthenticateMw<S> {
    service: S,
    cfg: Arc<OciConfig>,
}

impl<S, B> Service<ServiceRequest> for OciWwwAuthenticateMw<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let cfg = self.cfg.clone();
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut resp = fut.await?;
            if resp.status() == actix_web::http::StatusCode::UNAUTHORIZED {
                let header = format!(
                    "Bearer realm=\"https://{service}/auth/token\",service=\"{service}\"",
                    service = cfg.service
                );
                resp.headers_mut().insert(
                    actix_web::http::header::WWW_AUTHENTICATE,
                    actix_web::http::header::HeaderValue::from_str(&header).unwrap(),
                );
            }
            Ok(resp.map_into_left_body())
        })
    }
}
```

- [ ] **Step 2: Export + wire into OCI App in `main.rs`**

In `middleware/mod.rs`:

```rust
pub mod oci_www_authenticate;
pub use oci_www_authenticate::OciWwwAuthenticate;
```

In `main.rs` inside the OCI `App::new()`:

```rust
.wrap(a8n_api::middleware::OciWwwAuthenticate { cfg: Arc::new(cfg_oci.clone()) })
```

- [ ] **Step 3: Verify with curl**

```bash
just up
curl -i http://localhost:18081/v2/
```

Expected: `HTTP/1.1 401 Unauthorized` includes
`WWW-Authenticate: Bearer realm="https://registry.example.com/auth/token",service="registry.example.com"`.

- [ ] **Step 4: Commit**

```bash
git add api/src/middleware/oci_www_authenticate.rs api/src/middleware/mod.rs api/src/main.rs
git commit -m "feat(oci-registry): inject WWW-Authenticate on OCI 401 responses"
```

---

## Task 23: Integration test — happy-path pull

**Files:**
- Modify: `api/src/handlers/oci_registry.rs` (append an integration test module gated on `DATABASE_URL`)

- [ ] **Step 1: Write the test**

Append at the end of `api/src/handlers/oci_registry.rs`:

```rust
#[cfg(test)]
#[cfg(feature = "__integration")]
mod integration {
    // Only compiled when DATABASE_URL is set — behave like the download-proxy tests.
    // Use wiremock for the Forgejo registry upstream; build an in-process Actix App
    // that mirrors the production OCI App wiring; drive it with
    // `actix_web::test::init_service`.

    use super::*;
    use actix_web::{test, web, App};
    use sha2::{Digest, Sha256};
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn seed_pullable_app(pool: &PgPool, owner: &str, name: &str, tag: &str) -> uuid::Uuid {
        // Inserts an active application with oci_image_* populated.
        let id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO applications (id, slug, display_name, is_active,
                oci_image_owner, oci_image_name, pinned_image_tag)
             VALUES ($1, $2, $3, true, $4, $5, $6)"
        )
        .bind(id)
        .bind(format!("app-{}", id))
        .bind("App")
        .bind(owner)
        .bind(name)
        .bind(tag)
        .execute(pool).await.unwrap();
        id
    }

    #[sqlx::test]
    async fn happy_path_manifest_and_blob(pool: PgPool) {
        let server = MockServer::start().await;
        let manifest_body = br#"{"mediaType":"application/vnd.oci.image.manifest.v1+json","config":{"digest":"sha256:cfg","size":10},"layers":[{"digest":"sha256:lay","size":20}]}"#.to_vec();
        let blob_body = b"helloworld".to_vec();
        let blob_digest = format!("sha256:{}", hex::encode(Sha256::digest(&blob_body)));

        Mock::given(method("GET"))
            .and(path_regex("/v2/acme/app/manifests/v1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(manifest_body.clone())
                    .insert_header("Content-Type", "application/vnd.oci.image.manifest.v1+json")
                    .insert_header("Docker-Content-Digest", "sha256:manifest"),
            )
            .mount(&server).await;

        Mock::given(method("GET"))
            .and(path_regex("/v2/acme/app/blobs/.+"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(blob_body.clone())
                    .insert_header("Content-Type", "application/octet-stream"),
            )
            .mount(&server).await;

        let app_id = seed_pullable_app(&pool, "acme", "app", "v1").await;
        let slug = format!("app-{}", app_id);

        // Wire an in-process App with the same services as production.
        let client = Arc::new(ForgejoRegistryClient::new(server.uri(), "tok".into()));
        let manifest_cache = Arc::new(ManifestCache::new(300));
        let blob_cache = Arc::new(BlobCache::new(
            client.clone(),
            &format!("/tmp/oci-test-{}", uuid::Uuid::new_v4()),
            10_000_000,
            pool.clone(),
        ));
        blob_cache.ensure_dir().await.unwrap();
        let limiter = Arc::new(OciLimiter::new(2, 100));
        // Token service + test user + valid bearer:
        // ... build with a test JwtConfig, issue a scoped token, attach in Authorization header.

        // Build the App and run two requests:
        // 1. GET /v2/{slug}/manifests/v1 → 200, correct headers, body matches.
        // 2. GET /v2/{slug}/blobs/{blob_digest} → 200, body matches.
        // Verify the audit rows present after: OciPullRequested, OciPullCompleted.
    }
}
```

> The skeleton above leaves the full Actix `test::init_service` wiring as a concrete task the executor must complete. Model it on any existing integration test under `api/src/handlers/*.rs` that uses `actix_web::test`. The download-proxy plan already set this pattern up — cross-reference `handlers/download.rs` tests for the seeding + bearer-token plumbing idiom.

- [ ] **Step 2: Run**

Run: `DATABASE_URL=postgres://... docker compose -f compose.dev.yml exec -e DATABASE_URL api cargo test happy_path_manifest_and_blob --lib -- --nocapture`
Expected: PASS (requires the integration feature flag / DB).

- [ ] **Step 3: Commit**

```bash
git add api/src/handlers/oci_registry.rs
git commit -m "test(oci-registry): happy-path integration test (manifest + blob)"
```

---

## Task 24: Admin: accept OCI fields in `update_application` + invalidate caches

**Files:**
- Modify: `api/src/handlers/admin.rs`

- [ ] **Step 1: Accept new fields in the request body**

Find the struct used by the admin `update_application` handler (grep for `UpdateApplicationRequest` or similar) and add:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub oci_image_owner: Option<Option<String>>,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub oci_image_name: Option<Option<String>>,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub pinned_image_tag: Option<Option<String>>,
```

(Match the `Option<Option<String>>` nullable-update pattern the existing `forgejo_*` fields use, or plain `Option<String>` if that's the existing convention — be consistent.)

- [ ] **Step 2: Pass values to `ApplicationRepository::update`**

In the handler body, forward the new fields into the `UpdateApplication` struct you already build.

- [ ] **Step 3: Invalidate on pin change**

After the update, compare the old `pinned_image_tag` to the new. When it changed (including from/to `None`), fire:

```rust
if old_pinned_image_tag != new_pinned_image_tag {
    if let Some(mc) = manifest_cache.as_ref().as_ref() {
        mc.invalidate_app(app_id).await;
    }
    if let Some(bc) = blob_cache.as_ref().as_ref() {
        // Orphan sweep — best-effort, spawn so admin request isn't blocked.
        let bc = bc.clone();
        let pool = pool.clone();
        tokio::spawn(async move {
            if let Ok(keep) = collect_all_referenced_digests(&pool).await {
                let _ = bc.sweep_orphans(&keep).await;
            }
        });
    }
}
```

Where `collect_all_referenced_digests` is a helper you add at the bottom of `handlers/admin.rs`:

```rust
async fn collect_all_referenced_digests(pool: &PgPool) -> Result<Vec<String>, AppError> {
    // Walk every pullable application's pinned manifest (best-effort,
    // manifest-cache first, else re-fetch). For this first version,
    // return an empty vec — LRU eviction handles steady-state. Populate
    // in a follow-up if disk reclamation pressure requires it.
    Ok(Vec::new())
}
```

(Keeping this cheap first pass — the design spec noted the orphan sweep as best-effort with LRU as the backstop. An empty keep-set would wipe everything; do NOT call `sweep_orphans(&[])`. Guard with `if keep.is_empty() { return; }` inside the spawned task.)

- [ ] **Step 4: Write the test**

Append to the admin test module:

```rust
#[sqlx::test]
async fn admin_update_sets_oci_fields_and_invalidates_manifest_cache(pool: PgPool) {
    // ... construct an admin actor, seed an app, call the admin handler twice:
    // first sets oci fields, second changes pinned_image_tag.
    // Assert manifest cache is invalidated for that app on the second call.
}
```

(Mirror existing admin test plumbing — there are already tests that drive the admin update handler.)

- [ ] **Step 5: Run the test**

Run: `docker compose -f compose.dev.yml exec api cargo test admin_update_sets_oci_fields --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add api/src/handlers/admin.rs
git commit -m "feat(oci-registry): admin update accepts oci fields; invalidates caches on pin change"
```

---

## Task 25: Admin refresh endpoint — `POST /v1/admin/applications/{slug}/oci/refresh`

**Files:**
- Create: `api/src/handlers/admin_oci.rs`
- Modify: `api/src/handlers/mod.rs`, `api/src/routes/admin.rs` (register the route)

- [ ] **Step 1: Write the handler**

```rust
//! Admin-only: manually refresh OCI caches for an app.

use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use std::sync::Arc;

use crate::errors::AppError;
use crate::middleware::AdminUser;
use crate::repositories::ApplicationRepository;
use crate::responses::{get_request_id, success_no_data};
use crate::services::{BlobCache, ForgejoRegistryClient, ManifestCache};

pub async fn refresh_oci(
    req: HttpRequest,
    _admin: AdminUser,
    path: web::Path<String>,
    pool: web::Data<PgPool>,
    client: web::Data<Option<Arc<ForgejoRegistryClient>>>,
    manifest_cache: web::Data<Option<Arc<ManifestCache>>>,
) -> Result<HttpResponse, AppError> {
    let slug = path.into_inner();
    let request_id = get_request_id(&req);
    let client = client.get_ref().as_ref()
        .ok_or_else(|| AppError::not_found("OCI registry disabled"))?;
    let mc = manifest_cache.get_ref().as_ref()
        .ok_or_else(|| AppError::not_found("OCI registry disabled"))?;

    let app = ApplicationRepository::find_active_by_slug(pool.get_ref(), &slug).await?
        .ok_or_else(|| AppError::not_found("Application"))?;
    if !app.is_pullable() {
        return Err(AppError::not_found("Application not pullable"));
    }
    mc.invalidate_app(app.id).await;

    // Best-effort: re-fetch the pinned tag's manifest to warm the cache.
    let owner = app.oci_image_owner.as_deref().unwrap();
    let name  = app.oci_image_name.as_deref().unwrap();
    let tag   = app.pinned_image_tag.as_deref().unwrap();
    let _ = client.get_manifest(
        owner, name, tag,
        "application/vnd.oci.image.manifest.v1+json, application/vnd.oci.image.index.v1+json",
    ).await;

    Ok(success_no_data(request_id))
}
```

- [ ] **Step 2: Export + register the route**

In `handlers/mod.rs`: `pub mod admin_oci;`

In `routes/admin.rs`: add the route under the existing admin scope:

```rust
.route(
    "/applications/{slug}/oci/refresh",
    web::post().to(crate::handlers::admin_oci::refresh_oci),
)
```

- [ ] **Step 3: Write a test**

In `handlers/admin_oci.rs` add a `#[cfg(test)] mod tests` block exercising the handler via `actix_web::test::init_service` with a seeded admin and pullable app, plus a wiremock for the manifest refetch. Assert 200 + manifest cache is invalidated + wiremock saw the refetch request.

- [ ] **Step 4: Run**

Run: `docker compose -f compose.dev.yml exec api cargo test refresh_oci --lib -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add api/src/handlers/admin_oci.rs api/src/handlers/mod.rs api/src/routes/admin.rs
git commit -m "feat(oci-registry): admin refresh-oci endpoint"
```

---

## Task 26: Frontend — types + admin form fields

**Files:**
- Modify: `frontend/src/types/index.ts`, `frontend/src/pages/admin/AdminApplicationsPage.tsx` + `*.test.tsx`, `frontend/src/api/admin.ts` (or wherever `updateApplication` lives)

- [ ] **Step 1: Add fields to the `Application` TS type**

In `frontend/src/types/index.ts`:

```ts
oci_image_owner: string | null;
oci_image_name: string | null;
pinned_image_tag: string | null;
```

(Add next to the `forgejo_*` fields.)

- [ ] **Step 2: Add the three form fields + state + submit**

In `AdminApplicationsPage.tsx`, next to the existing Forgejo block, add a new **OCI Image** section with three `<Input>`s bound to state. Extend the save handler to include the three fields in the PATCH payload.

- [ ] **Step 3: Extend the API client**

In the admin update call (e.g., `frontend/src/api/admin.ts`), include the three fields in the `updateApplication` body type.

- [ ] **Step 4: Update the test**

In `AdminApplicationsPage.test.tsx`, add a test that types into the three inputs, clicks save, and asserts the PATCH mock was called with all three new fields.

- [ ] **Step 5: Run**

Run: `docker compose -f compose.dev.yml exec frontend bun run test:run src/pages/admin/AdminApplicationsPage.test.tsx`
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/types/index.ts frontend/src/pages/admin/AdminApplicationsPage.tsx \
        frontend/src/pages/admin/AdminApplicationsPage.test.tsx frontend/src/api/admin.ts
git commit -m "feat(oci-registry): admin form fields for oci image config"
```

---

## Task 27: Compose — volume, ports, env

**Files:**
- Modify: `compose.yml`, `compose.dev.yml`

- [ ] **Step 1: Add the volume**

In both compose files, add under `volumes:`:

```yaml
oci_cache:
  name: a8n-tools-oci-cache   # prod
# in compose.dev.yml:
#  name: saas-oci-cache-${USER}
```

- [ ] **Step 2: Mount into the `api` service**

Under `services.api.volumes:` in both files:

```yaml
- oci_cache:/var/cache/a8n-oci
```

- [ ] **Step 3: Expose the registry port (dev only)**

In `compose.dev.yml`, under `services.api.ports:` add:

```yaml
- "18081:18081"
```

- [ ] **Step 4: Plumb the eight env vars**

In both files under `services.api.environment:`:

```yaml
OCI_REGISTRY_ENABLED: ${OCI_REGISTRY_ENABLED:-false}
OCI_REGISTRY_PORT: ${OCI_REGISTRY_PORT:-18081}
OCI_REGISTRY_SERVICE: ${OCI_REGISTRY_SERVICE:-registry.example.com}
OCI_BLOB_CACHE_DIR: /var/cache/a8n-oci
OCI_BLOB_CACHE_MAX_BYTES: ${OCI_BLOB_CACHE_MAX_BYTES:-53687091200}
OCI_MANIFEST_CACHE_TTL_SECS: ${OCI_MANIFEST_CACHE_TTL_SECS:-300}
OCI_CONCURRENT_MANIFESTS_PER_USER: ${OCI_CONCURRENT_MANIFESTS_PER_USER:-2}
OCI_PULLS_PER_USER_PER_DAY: ${OCI_PULLS_PER_USER_PER_DAY:-50}
OCI_TOKEN_TTL_SECS: ${OCI_TOKEN_TTL_SECS:-900}
```

- [ ] **Step 5: Verify dev stack comes up**

Run: `just down && just up && docker compose -f compose.dev.yml ps`
Expected: `api` healthy. `docker compose -f compose.dev.yml logs api | grep -i oci` shows the OCI init log line. With `OCI_REGISTRY_ENABLED=true` set in `.env`, port 18081 reachable via `curl -i http://localhost:18081/v2/`.

- [ ] **Step 6: Commit**

```bash
git add compose.yml compose.dev.yml
git commit -m "feat(oci-registry): compose volume + env plumbing + registry port"
```

---

## Task 28: Documentation — user + dev guide

**Files:**
- Create: `docs/oci-registry.md`
- Modify: `CLAUDE.md` (add feature-flag section)

- [ ] **Step 1: Write `docs/oci-registry.md`**

Model it on `docs/forgejo-download-proxy.md`. Sections:

- **For users (admins + members)** — how to `docker login` + `docker pull` using a8n credentials; what to tell members; how admins configure `oci_image_*` fields.
- **For AI / developers**: architecture summary (auth flow diagram, pull-flow module table like the download-proxy doc has), config table, caveats.

Keep this ~120 lines max. Reference the spec (`docs/superpowers/specs/2026-04-16-oci-registry-design.md`) and plan (this file) for the deep details.

- [ ] **Step 2: Add feature-flag note to `CLAUDE.md`**

Under the existing "Feature Flags" section, add:

```markdown
### OCI registry
Gated behind `OCI_REGISTRY_ENABLED=true` + `FORGEJO_BASE_URL` + `FORGEJO_API_TOKEN`.
Second HTTP server at `OCI_REGISTRY_PORT` (default 18081) exposes an OCI-compliant
read-only registry. Members `docker login` with their a8n credentials and
`docker pull <registry>/<app-slug>:<pinned-tag>`. Blobs cached on disk at
`OCI_BLOB_CACHE_DIR` (volume `oci_cache`). See `docs/oci-registry.md`.
```

- [ ] **Step 3: Commit**

```bash
git add docs/oci-registry.md CLAUDE.md
git commit -m "docs(oci-registry): user + dev documentation"
```

---

## Task 29: Integration tests — edge cases

**Files:**
- Modify: `api/src/handlers/oci_registry.rs` (extend the integration module added in Task 23)

- [ ] **Step 1: Add these tests, each a separate `#[sqlx::test]`**

Each follows the same wiremock pattern as the happy-path test. Before writing, re-read the design spec's testing section for the exact expectations.

- `scope_mismatch_denies` — token for slug A used on slug B → `403` + `DENIED`.
- `cross_audience_token_rejected` — valid API JWT used as bearer → `401`.
- `rate_limit_denies_after_concurrency_cap` — exceed `concurrent_manifests_per_user` → `429` + `OciPullDeniedRateLimit` audit row.
- `daily_cap_denies_with_retry_after` — exceed `pulls_per_user_per_day` → `429` with `Retry-After`.
- `digest_mismatch_returns_502` — wiremock returns blob bytes that don't hash to the requested digest → `502`, no cache insert.
- `push_verbs_return_405` — `POST /v2/<slug>/blobs/uploads/` and `PUT /v2/<slug>/manifests/<tag>` each return `405 UNSUPPORTED`.
- `manifest_cache_hit_skips_upstream` — two pulls, wiremock `.expect(1)` on manifest endpoint; counts match after invalidation.
- `tag_change_invalidates_manifest_cache` — pull once; update `pinned_image_tag`; next pull goes upstream.
- `feature_disabled_omits_port` — with `OCI_REGISTRY_ENABLED=false`, assert (via a unit-level check of the `main::build_oci_server()` branch) that no registry server is created.

- [ ] **Step 2: Run the whole integration suite**

Run: `DATABASE_URL=postgres://... docker compose -f compose.dev.yml exec -e DATABASE_URL api cargo test oci_registry::integration --lib -- --nocapture`
Expected: all PASS.

- [ ] **Step 3: Commit**

```bash
git add api/src/handlers/oci_registry.rs
git commit -m "test(oci-registry): scope, audience, rate-limit, digest, push, cache, flag edge cases"
```

---

## Task 30: Manual smoke test against a real Forgejo instance

**Files:** n/a (ops task)

- [ ] **Step 1: Configure dev env**

In `.env`:

```
OCI_REGISTRY_ENABLED=true
FORGEJO_BASE_URL=https://<your-forgejo-host>
FORGEJO_API_TOKEN=<token-with-read-access>
OCI_REGISTRY_SERVICE=registry.example.localhost:18081
```

- [ ] **Step 2: Seed an application**

Via admin UI (`/admin/applications`), pick an existing app and fill in all three OCI fields (owner, name, tag) pointing at a real image in your Forgejo instance.

- [ ] **Step 3: `docker login`**

```bash
docker login registry.example.localhost:18081
Username: member@example.com
Password: ********
```

Expected: `Login Succeeded`. Verify `oci_login_succeeded` audit row.

- [ ] **Step 4: `docker pull`**

```bash
docker pull registry.example.localhost:18081/<app-slug>:<tag>
```

Expected: pull completes. Check:
- `/var/cache/a8n-oci/` contains one file per layer + manifest blob (if manifest was dereffed).
- `audit_logs` rows: `oci_pull_requested`, `oci_pull_completed`.
- `oci_blob_cache` and `oci_pull_daily_counts` tables have rows.

- [ ] **Step 5: Admin tag change smoke**

Change `pinned_image_tag` to another real tag via admin UI. Re-pull with new tag. Expected: succeeds; old-tag manifest cache is cleared (next request for old tag returns `404 MANIFEST_UNKNOWN` since it's no longer the pinned tag).

- [ ] **Step 6: Commit if any fixes emerged**

(Only commit if testing surfaced needed code changes; otherwise this task is a no-op commit.)

---

## Self-Review (run before marking plan done)

**Spec coverage:**
- [x] Registry topology (second HttpServer, feature flag) — Task 21
- [x] `/v2/` version probe — Task 19/20
- [x] `/auth/token` bearer issuance — Task 18
- [x] Manifest + blob HEAD/GET — Task 19
- [x] Push verbs → 405 — Task 19/20
- [x] `WWW-Authenticate` header on 401s — Task 22
- [x] Cross-audience protection on JWT — Task 15/17
- [x] Membership re-check on every request — Task 17/18
- [x] Scope enforcement — Task 17/19
- [x] ManifestCache TTL + invalidation — Task 13/24/25
- [x] BlobCache single-flight + digest verify + LRU — Task 14
- [x] Rate limiting (concurrency + daily) — Task 16/19
- [x] Separate counter table — Task 3/12
- [x] OCI error envelope — Task 8/9
- [x] Audit actions — Task 6, fired in Tasks 18/19
- [x] Admin OCI fields + invalidation — Task 4/5/24
- [x] Admin refresh endpoint — Task 25
- [x] Frontend admin form — Task 26
- [x] Config env vars — Task 7/27
- [x] Integration tests (happy + edge cases) — Task 23/29
- [x] Docs — Task 28

**Placeholder scan:** All code steps include code. Two tasks (17 step 3, 23 step 1, 29 step 1) contain scaffolded tests the executor must flesh out against existing patterns in the repo — these reference the exact sibling files to model from. Acceptable given the breadth.

**Type consistency:**
- `ForgejoRegistryClient` ↔ `RegistryError` — consistent.
- `OciError` variants ↔ `ResponseError` impl ↔ handler returns — consistent.
- `OciBearerUser` has `claims`; extractor must also populate `email` + `role` (noted in Task 19 callout) — executor must go back to Task 17 to add those fields. Noting here so it's caught before wiring the audit calls.
- `RegistryTokenClaims` + `REGISTRY_AUDIENCE` exported and used consistently.

**Remaining risk:** Task 19's audit calls use empty `""` placeholders for email/role — Task 17 enrichment must happen before Task 19 lands. Executor: when you reach Task 19 and see the `with_actor(user.claims.sub, "", "")` pattern, go back and extend `OciBearerUser` per the callout, then pass `&user.email, &user.role` through.
