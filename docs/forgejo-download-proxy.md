# Forgejo Download Proxy

Branch: `feat/forgejo-proxy`

Streams private Forgejo release assets to logged-in members through the
platform API, with an on-disk SHA-256 cache, LRU eviction, per-user rate
limiting, and audit logging. Source material: `docs/superpowers/specs/2026-04-15-forgejo-download-proxy-design.md`
and `docs/superpowers/plans/2026-04-15-forgejo-download-proxy.md`.

## For users (admins + members)

### Members

- A new **Downloads** page (`/dashboard/downloads`) lists every active,
  downloadable application grouped by app.
- Each application detail page now has a **Downloads** section showing
  the pinned release's assets.
- Clicking *Download* streams the file through the platform (the browser
  saves it via a `blob:` URL — no cross-host navigation).
- Non-members see an *Upgrade to access* CTA instead of the button.

Possible toasts:

- *You already have downloads in progress, please wait.* — per-user
  concurrency limit hit (HTTP 429).
- *Daily download limit reached. Try again in ~Nh.* — per-user daily cap
  hit (HTTP 429, with `Retry-After`).
- *Download source unavailable. Please try again later.* — Forgejo
  returned an error (HTTP 502).

### Admins

Applications now have three Forgejo-related fields in the admin edit
form (`/admin/applications`):

- **Forgejo Owner** — the user or organization that owns the repo.
- **Forgejo Repo** — the repository name.
- **Pinned Release Tag** — which release tag to serve.

When the pinned tag changes, the platform invalidates its release
metadata cache and the on-disk blobs for the previous tag.

A manual refresh endpoint exists at
`POST /v1/admin/applications/{slug}/downloads/refresh` to re-fetch the
release metadata on demand.

An application only appears in the downloads UI when owner, repo, and
tag are all set AND the feature flag (`FORGEJO_BASE_URL`) is configured.

## For AI / developers

### Architecture summary

Request flow for `GET /v1/applications/{slug}/downloads/{asset_name}`:

```
MemberUser auth
  └─ DownloadLimiter.acquire (concurrency + daily-count check)
     └─ ReleaseCache.get (Forgejo release metadata, moka TTL)
        └─ DownloadCache.get_or_fetch (disk-backed, SHA-256 addressed)
           └─ stream file → client (guard held until EOF)
```

Key modules:

| File | Purpose |
| --- | --- |
| `api/src/services/forgejo.rs` | HTTP client for Forgejo API. Validates asset URL host/port/scheme against `FORGEJO_BASE_URL` before forwarding the API token. |
| `api/src/services/release_cache.rs` | moka TTL cache over `ForgejoClient::get_release`. |
| `api/src/services/download_cache.rs` | On-disk cache keyed by `(app_id, release_tag, asset_name)`. Single-flight via `Arc<OnceCell>`. Atomic rename + fsync. Async LRU eviction. |
| `api/src/services/download_limiter.rs` | Per-user in-process concurrency map + Postgres-backed daily counter. RAII `DownloadGuard`. |
| `api/src/repositories/download_cache.rs` | `download_cache` table access. `upsert` returns the replaced SHA so callers can unlink orphaned blobs. |
| `api/src/repositories/download_daily_count.rs` | `download_daily_counts` table access (UTC day boundary). |
| `api/src/handlers/download.rs` | Member-facing download handlers + admin refresh. Audit events emitted from inside the stream's EOF / error branches (exactly-once via a flag). |
| `api/src/routes/downloads.rs` | Route registration. |
| `frontend/src/lib/downloads.ts` | `triggerDownload(url, filename)` — credentialed fetch → blob → anchor click. Handles 429/502 toasts. |
| `frontend/src/components/downloads/AppDownloadsSection.tsx` | Per-app downloads section. |
| `frontend/src/pages/dashboard/DownloadsPage.tsx` | Global downloads page. |

### Database migrations

- `api/migrations/*_add_forgejo_columns_to_applications.sql` adds
  `forgejo_owner`, `forgejo_repo`, `pinned_release_tag` to `applications`.
- `api/migrations/*_create_download_cache.sql` — disk-cache bookkeeping.
- `api/migrations/*_create_download_daily_counts.sql` — per-user daily
  counter, primary key `(user_id, day_utc)`.

### Audit actions

New `audit_logs.action` variants (in `api/src/models/audit.rs`):

- `DownloadRequested` — acquire succeeded; fired before streaming.
- `DownloadCompleted` — emitted on stream EOF (not before).
- `DownloadFailedUpstream` — Forgejo or mid-stream I/O failure.
- `DownloadDeniedRateLimit` — concurrency or daily cap denial.

### Config (environment variables)

All loaded in `api/src/config.rs` via `Config::from_env()`. Feature is
**disabled** if `FORGEJO_BASE_URL` or `FORGEJO_API_TOKEN` is unset —
handlers return 404 for the download routes in that state.

| Variable | Default | Purpose |
| --- | --- | --- |
| `FORGEJO_BASE_URL` | *(unset → disabled)* | Root of the Forgejo instance, e.g. `https://git.example.com`. Asset URLs are validated against this host/port/scheme before the API token is forwarded. |
| `FORGEJO_API_TOKEN` | *(unset → disabled)* | Personal access token sent as `Authorization: token <value>`. Needs read access to the repos you want to serve. |
| `DOWNLOAD_CACHE_DIR` | `/var/cache/a8n-downloads` | Directory for SHA-256-named blobs. Must be writable. |
| `DOWNLOAD_CACHE_MAX_BYTES` | `10737418240` (10 GiB) | Soft cap. When exceeded, LRU eviction runs async after each successful fetch. |
| `DOWNLOAD_CONCURRENCY_PER_USER` | `2` | Simultaneous in-flight downloads per user. In-process only — single-instance deployments only. |
| `DOWNLOAD_DAILY_LIMIT_PER_USER` | `50` | Downloads per UTC day per user. Rolls back the counter if the cap is exceeded between increment and acquire. |
| `FORGEJO_RELEASE_CACHE_TTL_SECS` | `300` | moka TTL for release metadata. |

Compose changes (`compose.dev.yml`, `compose.yml`):

- New named volume `downloads_cache` → `/var/cache/a8n-downloads`
  (dev: `saas-downloads-cache-${USER}`).
- All seven env vars above are plumbed through to the `api` service.

### Caveats / known limits

- **Single-instance only.** `DownloadLimiter.inflight` is an in-process
  map. Multi-instance deploys need a Postgres-backed replacement — a
  TODO is noted in `api/src/services/download_limiter.rs`.
- **Source-code archives are not exposed.** `RawRelease` only
  deserializes uploaded `assets[]`; Forgejo's auto-generated
  `tarball_url` / `zipball_url` are ignored.
- **Audit timing.** `DownloadRequested` fires before the stream starts;
  `DownloadCompleted` / `DownloadFailedUpstream` fire from inside the
  streaming body. Client aborts after a clean EOF still record as
  completed — by design.
- **Rate-limit counter is best-effort on rollback.** If decrementing the
  daily counter fails after a cap-exceeded denial, the counter may be
  off by one until the day rolls over. The concurrency slot is released
  unconditionally.

### Testing

- `cargo test` for unit tests.
- `DATABASE_URL=… cargo test -- download` to exercise the integration
  tests in `api/src/handlers/download.rs` (otherwise skipped).
- Frontend: `bun run test:run src/components/downloads src/pages/dashboard/DownloadsPage.test.tsx`.
