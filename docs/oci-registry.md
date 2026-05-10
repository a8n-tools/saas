# OCI Registry Proxy

Branch: `feat/oci-registry`

Exposes a read-only, OCI Distribution Spec v1.1–compliant container
registry that proxies private Forgejo images to logged-in a8n members.
A second HTTP server runs on its own port and subdomain (e.g.
`oci.example.com`). Blobs are cached on disk, addressed by
SHA-256. Source material: `docs/superpowers/specs/2026-04-16-oci-registry-design.md`
and `docs/superpowers/plans/2026-04-16-oci-registry.md`.

## For users (admins + members)

### Members

- `docker login oci.example.com` with your a8n email + password.
  (TOTP is not currently supported on registry login — use a browser
  for first-time password recovery.)
- `docker pull oci.example.com/<app-slug>:<pinned-tag>` pulls the
  image an admin has configured for that app.
- Only active members can pull. Lapsed subscribers get `401 UNAUTHORIZED`.
- Push is not supported. `POST`/`PUT`/`PATCH`/`DELETE` under `/v2/*`
  returns `405 UNSUPPORTED`.

Possible error envelopes (OCI-spec JSON):

- `DENIED` — the bearer token's scope doesn't match the requested repo.
- `MANIFEST_UNKNOWN` — the pinned tag doesn't resolve upstream.
- `BLOB_UNKNOWN` — the referenced layer is missing or has a bad digest.
- HTTP `429` with `Retry-After` — per-user rate limit exceeded.

### Admins

Applications now have three OCI-related fields in the admin edit form
(`/admin/applications`):

- **OCI Image Owner** — container-registry owner on Forgejo (may differ
  from the release-download owner).
- **OCI Image Name** — image/repository name.
- **Pinned Image Tag** — which tag to serve.

When the pinned tag changes, the platform invalidates the in-memory
manifest cache. On-disk blob cleanup is handled passively by `BlobCache`'s
async LRU eviction — a dedicated orphan sweep is not implemented.

A manual refresh endpoint exists at
`POST /v1/admin/applications/{slug}/oci/refresh` — it invalidates the
manifest cache and warms it by re-fetching + caching the pinned tag.

An application only appears as pullable when owner, name, and tag are
all set, the app is active, AND the feature flag
(`OCI_REGISTRY_ENABLED=true` plus `FORGEJO_BASE_URL` + `FORGEJO_API_TOKEN`)
is configured.

## For AI / developers

### Architecture summary

Two servers run in the same process (see `api/src/main.rs`):

- **Primary API** on `APP_PORT` — unchanged platform behavior.
- **OCI Registry** on `OCI_REGISTRY_PORT` — OCI-spec endpoints only.

Pull flow:

```
docker pull
  └─ GET /v2/                     → 401 + WWW-Authenticate (realm=/auth/token)
     └─ GET /auth/token           (Basic auth: email + password)
        → JWT, aud="registry", scope="repository:<slug>:pull"
  └─ GET /v2/<slug>/manifests/<ref>   (Bearer)
     ├─ OciBearerUser extractor     (verify + reload user)
     ├─ OciLimiter.acquire          (concurrency + daily cap)
     ├─ ManifestCache.get           (moka TTL)
     └─ ForgejoRegistryClient.get_manifest → cache insert
  └─ GET /v2/<slug>/blobs/<digest>
     ├─ OciBearerUser extractor
     └─ BlobCache.get_or_fetch      (disk, SHA-256-addressed, single-flight)
        └─ stream file → docker
```

Key modules:

| File | Purpose |
| --- | --- |
| `api/src/services/forgejo_registry.rs` | Upstream registry client. Token forwarded only to hosts matching `FORGEJO_BASE_URL`. Redirects disabled. |
| `api/src/services/oci_token.rs` | Issues + verifies `aud="registry"` JWTs. Distinct audience from primary-API tokens (rejected cross-use). |
| `api/src/services/manifest_cache.rs` | moka TTL cache keyed by `(app_id, reference)`. |
| `api/src/services/blob_cache.rs` | On-disk cache keyed by digest. Single-flight via `Arc<OnceCell>`. Atomic rename + fsync. Async LRU eviction. Validates `sha256:<64 hex>` format to block path traversal. |
| `api/src/services/oci_limiter.rs` | Per-user manifest concurrency + Postgres daily pull counter. RAII `OciPullGuard`. |
| `api/src/repositories/oci_blob_cache.rs` | `oci_blob_cache` table access. |
| `api/src/repositories/oci_pull_daily_counts.rs` | `oci_pull_daily_counts` table. |
| `api/src/middleware/oci_auth.rs` | `OciBearerUser` extractor: verify token, reload user, re-check membership, enforce scope in handlers. |
| `api/src/middleware/oci_www_authenticate.rs` | Injects `WWW-Authenticate: Bearer …` on any 401 from the OCI server. |
| `api/src/handlers/oci_auth.rs` | `GET /auth/token` — Basic-auth → registry JWT. Timing-safe user-not-found path via cached dummy Argon2 hash. |
| `api/src/handlers/oci_registry.rs` | `/v2/` probe, manifest + blob handlers, push-verb 405s. |
| `api/src/handlers/admin_oci.rs` | Admin refresh endpoint. |
| `api/src/routes/oci.rs` | Route config for the OCI `App` (not mounted under `/v1`). |

### Database migrations

- `*_add_oci_columns_to_applications.sql` adds `oci_image_owner`,
  `oci_image_name`, `pinned_image_tag` to `applications`.
- `*_create_oci_blob_cache.sql` — SHA-256-addressed blob bookkeeping.
- `*_create_oci_pull_daily_counts.sql` — per-user daily pull counter,
  primary key `(user_id, day_utc)`.

### Audit actions

New `audit_logs.action` variants (in `api/src/models/audit.rs`):

- `OciLoginFailed` — `/auth/token` rejected (bad creds or not a member).
- `OciPullRequested` — token verified, scope matched, limiter acquired.
- `OciPullCompleted` — manifest/blob streamed successfully.
- `OciPullDeniedScope` — token scope didn't match the requested slug.
- `OciPullDeniedRateLimit` — concurrency or daily cap denial.
- `OciPullFailedUpstream` — Forgejo registry returned an error.

### Config (environment variables)

All loaded in `api/src/config.rs`. Feature is **disabled** if
`OCI_REGISTRY_ENABLED` is false OR `FORGEJO_BASE_URL` /
`FORGEJO_API_TOKEN` are unset — in that state no second server is
spawned.

| Variable | Default | Purpose |
| --- | --- | --- |
| `OCI_REGISTRY_ENABLED` | `false` | Master switch. |
| `OCI_REGISTRY_PORT` | `18081` | Port for the OCI-only server. Front this with a dedicated subdomain. |
| `OCI_REGISTRY_SERVICE` | `oci.example.com` | `service` value advertised in `WWW-Authenticate` + embedded in tokens. |
| `OCI_BLOB_CACHE_DIR` | `/var/cache/a8n-oci` | Directory for SHA-256-named blobs. Must be writable. |
| `OCI_BLOB_CACHE_MAX_BYTES` | `53687091200` (50 GiB) | Soft cap. LRU eviction runs async after each successful fetch. |
| `OCI_MANIFEST_CACHE_TTL_SECS` | `300` | moka TTL for manifests. |
| `OCI_CONCURRENT_MANIFESTS_PER_USER` | `2` | Per-user in-flight manifest fetches. In-process map — single-instance only. |
| `OCI_PULLS_PER_USER_PER_DAY` | `50` | Per-user UTC-day pull cap. |
| `OCI_TOKEN_TTL_SECS` | `900` | Registry token lifetime. |

Compose changes (`compose.yml`, `compose.dev.yml`):

- New named volume `oci_cache` → `/var/cache/a8n-oci`
  (prod: `a8n-tools-oci-cache`, dev: `saas-oci-cache-${USER}`).
- Dev compose exposes `18081:18081` on the host and wires a Traefik router
  for `${USER}-oci.a8n.run` → container port `18081`.
- All nine env vars above are plumbed through to the `api` service.

**Prod compose deployment note.** `compose.yml` is a template — the `api`
service has `traefik.enable: false` by design. Deployments that want to
expose the registry must add their own Traefik router (or equivalent
ingress) that maps the registry hostname to the `api` container on
`OCI_REGISTRY_PORT` (default `18081`). If the feature flag is enabled
without a route in place, `docker login` and `docker pull` will simply
never reach the OCI server.

### Caveats / known limits

- **Single-instance only.** `OciLimiter.inflight` is an in-process map
  (same pattern as the download proxy). Multi-instance deploys need a
  Postgres-backed replacement — TODO noted in
  `api/src/services/oci_limiter.rs`.
- **Read-only.** Push verbs return `405 UNSUPPORTED`. The feature is
  a proxy for admin-curated images, not a general-purpose registry.
- **Admin-curated pin only.** Members cannot pull arbitrary tags — only
  whatever `pinned_image_tag` the admin has set for the slug.
- **No active orphan sweep.** On pin change we invalidate the in-memory
  manifest cache. Stale blobs on disk are reclaimed passively by
  `BlobCache`'s async LRU eviction when `OCI_BLOB_CACHE_MAX_BYTES` is
  reached. A reachability-based sweep would be additive work if disk
  pressure becomes a problem.
- **Password-only login.** Docker's `~/.docker/config.json` Basic-auth
  flow predates 2FA. Members with TOTP enabled still log in with their
  password here — TOTP is enforced on the web UI but not on
  `/auth/token`. An API-token flow is a future-work item.

### Testing

- `cargo test` for unit tests.
- `DATABASE_URL=… cargo test -- oci` for DB-backed integration tests
  (otherwise skipped).
- Frontend: `bun run test:run src/pages/admin/AdminApplicationsPage.test.tsx`.
