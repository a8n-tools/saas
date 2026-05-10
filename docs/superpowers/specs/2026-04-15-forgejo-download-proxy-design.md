# Forgejo Download Proxy — Design

**Date:** 2026-04-15
**Branch:** `feat/forgejo-proxy`

## Purpose

Distribute compiled binaries and container-image tarballs to members with active
membership. Files live in private Forgejo releases; the API proxies downloads
after verifying membership and recording an audit trail. Public Git + public CI
stay out of reach; distribution is gated.

## Scope

In scope:

- Schema changes on `applications` + new `download_cache` and
  `download_daily_counts` tables
- Member API: list downloads per app, global list, stream a specific asset
- Admin API + admin UI: configure `forgejo_owner`, `forgejo_repo`,
  `pinned_release_tag` per app, plus a manual "refresh release" action
- Forgejo client, in-memory release-metadata cache, on-disk file cache with LRU
  eviction, per-user concurrency + daily-count rate limiters
- Audit logging of every download attempt (success + denial + failure)
- Frontend: per-app downloads section and global `/downloads` page

Out of scope:

- `docker pull` from the Forgejo container registry (members consume container
  images as pre-exported `docker save` tarballs and run `docker load`)
- Pre-warming the cache when an admin pins a tag
- Multi-instance deployment of the API (see "Follow-ups")

## Audience & auth

Only logged-in users with active membership may download. Auth is the existing
JWT cookie via the `MemberUser` extractor. All downloads are browser-initiated.

## Data model

### `applications` (altered)

Add three nullable columns:

- `forgejo_owner TEXT`
- `forgejo_repo TEXT`
- `pinned_release_tag TEXT`

An application is "downloadable" when all three are non-null. The admin UI
enforces all-or-nothing on save.

### `download_cache` (new)

| column             | type        | notes                                    |
| ------------------ | ----------- | ---------------------------------------- |
| `id`               | UUID PK     |                                          |
| `application_id`   | UUID        | FK `applications(id) ON DELETE CASCADE`  |
| `release_tag`      | TEXT        |                                          |
| `asset_name`       | TEXT        |                                          |
| `content_sha256`   | TEXT        | also the filename on disk                |
| `size_bytes`       | BIGINT      |                                          |
| `content_type`     | TEXT        |                                          |
| `created_at`       | TIMESTAMPTZ |                                          |
| `last_accessed_at` | TIMESTAMPTZ |                                          |

Unique constraint on `(application_id, release_tag, asset_name)`.
LRU eviction query: `ORDER BY last_accessed_at ASC`.

### `download_daily_counts` (new)

| column    | type | notes                       |
| --------- | ---- | --------------------------- |
| `user_id` | UUID | PK part                     |
| `day`     | DATE | PK part (UTC)               |
| `count`   | INT  | incremented per attempt     |

### In-memory release-metadata cache

Not persisted. Keyed by `(application_id, pinned_release_tag)`. TTL 5 min
(configurable). Holds parsed Forgejo release JSON: asset list with
`asset_id`, `name`, `size`, `content_type`, Forgejo download URL.

### Audit log action codes

- `download_requested`
- `download_completed`
- `download_denied_membership`
- `download_denied_rate_limit`
- `download_failed_upstream`

## API

### Member endpoints (require `MemberUser`)

- `GET /v1/applications/{slug}/downloads`

  Returns the pinned release's assets:

  ```json
  {
    "release_tag": "v1.4.0",
    "assets": [
      {
        "asset_name": "rus-linux-x86_64.tar.gz",
        "size_bytes": 8123456,
        "content_type": "application/gzip",
        "download_url": "/v1/applications/rus/downloads/rus-linux-x86_64.tar.gz"
      }
    ]
  }
  ```

  - App slug not found → 404.
  - App exists but not downloadable → 200 with `{"release_tag": null, "assets": []}`.

- `GET /v1/downloads`

  Global list across all downloadable active apps. Same asset shape, grouped by
  app slug / display name / icon.

- `GET /v1/applications/{slug}/downloads/{asset_name}`

  Streams bytes. Headers: `Content-Type`, `Content-Length`,
  `Content-Disposition: attachment; filename="<asset_name>"`. Serves from disk
  cache; on miss, streams from Forgejo while populating the cache.

  - App not downloadable or asset not in pinned release → 404.
  - User over daily cap → 429 with `Retry-After: <seconds-until-UTC-midnight>`.
  - User over concurrency cap → 429 with error code
    `download_concurrency_limit`.
  - Forgejo upstream failure → 502.

### Admin endpoints (require `AdminUser`)

- `PATCH /v1/admin/applications/{slug}` — extended to accept
  `forgejo_owner`, `forgejo_repo`, `pinned_release_tag`. Changing
  `pinned_release_tag` invalidates all `download_cache` rows for the previous
  `(application_id, old_tag)` — DB rows deleted, disk files unlinked if no
  other row references the same `content_sha256`.

- `POST /v1/admin/applications/{slug}/downloads/refresh` — invalidates the
  in-memory release-metadata cache entry for the app and re-fetches it,
  returning the resolved asset list or the Forgejo error. No disk cache
  pre-warm.

## Services

- `services/forgejo.rs` — `ForgejoClient` wraps `reqwest`. Reads
  `FORGEJO_BASE_URL` + `FORGEJO_API_TOKEN` from config. Sends
  `Authorization: token <...>`. Methods:
  `get_release(owner, repo, tag) -> ReleaseMetadata`,
  `download_asset(asset_id) -> impl Stream<Bytes>`.

- `services/release_cache.rs` — in-memory TTL cache wrapping
  `ForgejoClient::get_release`. Default TTL 300 s. Exposes
  `invalidate(app_id, tag)` for the admin refresh endpoint.

- `services/download_cache.rs` — on-disk file cache.

  - `get_or_fetch(app, tag, asset) -> CachedFile`: returns an open `File` + the
    row metadata. On miss, streams Forgejo → tempfile, computes SHA-256
    on the way, `fsync`, atomically renames to
    `DOWNLOAD_CACHE_DIR/<content_sha256>`, upserts the `download_cache` row,
    and asynchronously calls `evict_lru_to_fit`.
  - `invalidate_app_tag(app_id, tag)`: deletes DB rows for that pair, unlinks
    any on-disk file whose SHA is no longer referenced.
  - `evict_lru_to_fit(max_bytes)`: while total `size_bytes` > cap, delete
    oldest-by-`last_accessed_at` row and unlink file if unreferenced.
  - Concurrent same-asset fetches are de-duped via an in-process
    `Arc<Mutex<HashMap<CacheKey, Arc<OnceCell<Result<...>>>>>>` so a second
    request waits on the first.

- `services/download_limiter.rs`

  - Per-user in-flight concurrency: `Arc<Mutex<HashMap<UserId, usize>>>`.
    Cap from `DOWNLOAD_CONCURRENCY_PER_USER` (default 2). Returns an RAII
    guard that decrements on drop.
  - Per-user daily count: `download_daily_counts` table.
    `INSERT ... ON CONFLICT (user_id, day) DO UPDATE SET count = count + 1
    RETURNING count`. Cap from `DOWNLOAD_DAILY_LIMIT_PER_USER` (default 50).
    Day boundary is UTC midnight.
  - `// TODO (follow-up):` replace the in-process concurrency map with a
    Postgres-row + heartbeat implementation so the API can run multi-instance.

- `repositories/download_cache.rs` — CRUD, upsert, LRU-ordered listing,
  tag-invalidation delete.

- `handlers/download.rs` — three member handlers + admin refresh.

## Request flow — member asset download

1. `MemberUser` extractor: authenticated + active membership, or 401/403.
2. Load application by slug. If not downloadable, 404.
3. `download_limiter` checks (concurrency then daily). On violation, emit
   `download_denied_rate_limit` audit and return 429.
4. Emit `download_requested` audit.
5. `release_cache.get_release(app)`; locate asset by name or 404.
6. `download_cache.get_or_fetch(...)`:
   - hit → `UPDATE last_accessed_at = now()`
   - miss → stream from Forgejo, verify, upsert row, schedule LRU eviction
7. Open the cached file, return `HttpResponse::Ok().streaming(...)` with
   the right headers.
8. On response completion: emit `download_completed`. RAII guard drops and
   decrements the concurrency counter.
9. On upstream/disk error: emit `download_failed_upstream`, return 502.

## Frontend

- **API client additions** (`frontend/src/api/`): `getAppDownloads(slug)`,
  `getAllDownloads()`. The actual file download is a plain `<a href>` to the
  streaming endpoint; the browser sends the auth cookie automatically.

- **Per-app downloads section** on the existing application detail page. Shows
  the pinned tag and an asset list (name, size, download button). When the app
  has no Forgejo config: muted "No downloads available" message. When the user
  has no active membership: render the list but swap download buttons for a
  gated CTA linking to membership upgrade.

- **Global `/downloads` page**, linked from primary nav. Groups assets by
  application: display name, icon, pinned tag, asset list. Same gating as
  above.

- **Admin app editor:** three new fields (`forgejo_owner`, `forgejo_repo`,
  `pinned_release_tag`) with all-or-nothing validation. "Test / refresh
  release" button calls the admin refresh endpoint and displays the resolved
  asset list or error.

- **Error UX:**
  - 429 concurrency → toast: "You already have downloads in progress, please
    wait."
  - 429 daily cap → toast with reset time.
  - 502 / network → toast: "Download failed, please try again."

## Configuration

New env vars (`api/src/config.rs`), all optional:

| var                              | default                      |
| -------------------------------- | ---------------------------- |
| `FORGEJO_BASE_URL`               | _unset disables the feature_ |
| `FORGEJO_API_TOKEN`              | _unset disables the feature_ |
| `DOWNLOAD_CACHE_DIR`             | `/var/cache/a8n-downloads`   |
| `DOWNLOAD_CACHE_MAX_BYTES`       | `10737418240` (10 GiB)       |
| `DOWNLOAD_CONCURRENCY_PER_USER`  | `2`                          |
| `DOWNLOAD_DAILY_LIMIT_PER_USER`  | `50`                         |
| `FORGEJO_RELEASE_CACHE_TTL_SECS` | `300`                        |

When the feature is disabled: member endpoints return 404, the admin form
shows a "Forgejo not configured" banner. Mirrors the existing Stripe/SMTP
disable-when-unconfigured pattern.

`compose.yml` / `compose.dev.yml`: add a named volume for the download cache,
mounted at `DOWNLOAD_CACHE_DIR` in the API container.

## Testing

**Rust unit tests:**

- `download_cache`: hit path, miss path (mocked Forgejo stream), SHA-256
  verification, LRU eviction ordering, tag-change invalidation with shared-SHA
  retention.
- `download_limiter`: concurrency RAII decrement on drop + on panic, daily
  counter rollover at UTC midnight.

**Rust integration tests** (wiremock for Forgejo):

- Member happy path (cache miss → hit).
- Non-member 403.
- App without Forgejo config: list returns `{assets: []}`; asset GET 404.
- Asset not in pinned release → 404.
- Concurrency cap → 429.
- Daily cap → 429 with correct `Retry-After`.
- Forgejo 5xx → 502.

**Frontend tests** (Vitest + MSW): per-app section, global downloads page, and
admin form — covering gated-vs-ungated states and 429/502 error toasts.

## Operational notes

- Disk cache must survive container restarts — use a named volume.
- Cache invalidation is tag-change + manual admin refresh. No TTL on files.
- First member download after a tag change repopulates the cache (no
  pre-warm).
- Audit-log volume scales with download attempts; the existing `audit_logs`
  table handles this.

## Follow-ups (tracked as `// TODO (follow-up):` comments in code)

- Replace in-process concurrency map with Postgres-row + heartbeat for
  multi-instance deployments.
- Optional pre-warming of the disk cache when an admin pins a new tag.
