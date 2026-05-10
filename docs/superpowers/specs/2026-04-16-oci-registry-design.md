# OCI Registry — Design

Date: 2026-04-16
Branch: `feat/oci-registry`
Status: Draft

## Problem

Members need to pull container images belonging to their applications. The
images live in Forgejo's OCI registry, which requires credentials members
don't have. The platform already proxies Forgejo release assets
(`docs/forgejo-download-proxy.md`); this feature extends that model to
OCI images so users can `docker pull` from a platform-hosted registry
using their a8n account.

## Goals

- Stand up an OCI Distribution Spec v1.1 compliant read-only registry at
  `registry.example.com` that proxies per-application pinned images from
  Forgejo.
- Authenticate via the OCI Bearer flow using existing a8n credentials
  (email + password) with an active-membership check on every request.
- Cache image layers on disk (immutable, SHA-256 addressed) to absorb
  repeat pulls and reduce Forgejo bandwidth.
- Rate-limit per user with a counter independent of the release download
  limits.
- Emit audit events parallel to the download-proxy pattern so admins can
  answer "who pulled what, when."

## Non-goals

- Push. The registry is read-only; push verbs return `405`.
- Personal access tokens / per-device credentials. Direct email+password
  is sufficient for v1; PATs can land as a follow-up.
- Multi-instance deployment. Like the download proxy, the concurrency
  limiter is in-process and single-instance only.
- Arbitrary Forgejo image access. Only images referenced by an
  application's pinned tag are pullable; no catalog endpoint.
- Source-archive style auto-generated images — irrelevant; images are
  always explicitly published.

## User-facing workflow

```
$ docker login registry.example.com
Username: member@example.com
Password: ********
Login Succeeded

$ docker pull registry.example.com/my-app:v1.4.2
v1.4.2: Pulling from my-app
...
Status: Downloaded newer image for registry.example.com/my-app:v1.4.2
```

The image reference uses the application's **slug** as the repository
name. The only pullable tag is the application's `pinned_image_tag`
(configured by admins); any other tag returns `404 MANIFEST_UNKNOWN`.
Admins configure the image via three new fields on the application edit
form: `oci_image_owner`, `oci_image_name`, `pinned_image_tag`.

Non-members (expired membership, inactive user) get `401` on every
request, forcing a re-login that will also fail. The browser UI does not
change; this feature is CLI-only.

## Architecture

### Process topology

The registry runs inside the existing `a8n-api` binary as a second
`HttpServer` bound to a new internal port (default `18081`). The
reverse proxy routes `registry.example.com` → `18081` and continues to
route `app.example.com` → `18080`. Two ports (rather than Host-header
routing on one port) keep the registry's middleware stack — Bearer-only
auth, no cookies, no CSRF — fully isolated from the web/API stack. The
second `HttpServer` shares the same `web::Data` services (Postgres pool,
Forgejo clients, caches) as the primary server.

The feature is **disabled** (second port is not bound) unless all of the
following are set:

- `FORGEJO_BASE_URL`
- `FORGEJO_API_TOKEN`
- `OCI_REGISTRY_ENABLED=true`

### Endpoints

All registry endpoints live under the secondary server:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/v2/` | API version check. `200` if authenticated; else `401` + `WWW-Authenticate`. |
| `GET` | `/auth/token` | Basic-auth → issues short-lived JWT bearer token. |
| `HEAD`/`GET` | `/v2/<slug>/manifests/<reference>` | Manifest by tag or digest (tag must equal `pinned_image_tag`). |
| `HEAD`/`GET` | `/v2/<slug>/blobs/<digest>` | Blob by SHA-256 digest. Supports `Range`. |
| Other verbs under `/v2/*` | `405 UNSUPPORTED` — push is not implemented. |

### Authentication flow

1. Client: `GET /v2/` with no auth.
2. Server: `401 Unauthorized` with
   `WWW-Authenticate: Bearer realm="https://registry.example.com/auth/token",service="registry.example.com"`.
3. Client: `GET /auth/token?service=registry.example.com&scope=repository:<slug>:pull`
   with `Authorization: Basic base64(email:password)`.
4. Server: verifies password via `AuthService::verify_password`, checks
   the user is active, checks the user has an active membership, and —
   if a `scope` is requested — checks the target application exists, is
   active, and has all three `oci_image_*` fields set.
5. On success: issues a JWT (`aud="registry"`, `sub=user_id`,
   `scope="repository:<slug>:pull"`, `iat`, `exp=iat+900`) signed with
   the existing platform JWT keypair. Emits `OciLoginSucceeded`. Returns
   `200 { "token": "...", "expires_in": 900, "issued_at": "..." }`.
6. On failure: emits `OciLoginFailed` (with a reason code field) and
   returns `401`.

Subsequent requests carry `Authorization: Bearer <jwt>`. The
`OciBearerUser` extractor:

- Validates signature, expiry, and `aud == "registry"`.
- Re-loads the user and membership on every request (single indexed
  query). A user whose membership lapsed after the token was issued is
  denied and must re-login.
- On handlers whose URL carries a `<slug>` (`/v2/<slug>/...`), the
  handler additionally requires the token's `scope` claim to match
  `repository:<slug>:pull`; else `403 DENIED`. `GET /v2/` has no slug
  and accepts any valid registry-audience token (including
  scope-less ones issued for a bare `service=registry.example.com`
  request).

### Pull flow

For `docker pull registry.example.com/<slug>:<tag>`:

1. **Manifest fetch** — `GET /v2/<slug>/manifests/<tag>`:
   a. `OciBearerUser` extractor authenticates + scope-checks.
   b. Handler enforces `<tag> == application.pinned_image_tag`; else
      `404 MANIFEST_UNKNOWN`.
   c. `OciLimiter.acquire(user_id)` — claims a concurrency slot and
      increments the daily pull counter. Denial → `429` with
      `Retry-After` for the daily-cap case. Emits `OciPullRequested`.
   d. `ManifestCache.get_or_fetch((app_id, tag))` — on miss,
      `ForgejoRegistryClient.get_manifest(owner, name, tag, accept_headers)`;
      client's `Accept` headers are passed through so Forgejo returns
      the right media type (image manifest vs. image index for
      multi-arch). Cached: raw bytes + `Content-Type` + `Docker-Content-Digest`.
   e. Returns manifest with `Content-Type`, `Docker-Content-Digest`,
      `Content-Length` headers. Emits `OciPullCompleted`.
2. **Blob fetches** — for each referenced digest, client issues
   `HEAD`/`GET /v2/<slug>/blobs/sha256:<digest>` in parallel:
   a. Extractor authenticates + scope-checks on each request.
   b. `BlobCache.get_or_fetch(digest, fetch_fn)`:
      - Cache hit: stream file from disk; bump `last_access_at`.
      - Miss: single-flight fetch from Forgejo to a `.partial` file,
        hashing on the fly, verifying the computed hash matches the
        requested digest. Mismatch → delete partial, return `502`,
        emit `OciPullFailedUpstream`. Match → atomic rename + fsync +
        insert row into `oci_blob_cache`, then stream to client.
   c. `Range` requests are served from disk after the blob is fully
      cached. On a cold-cache `Range` request, the first fetch
      populates the cache (full body); the client reissues with
      `Range` on its own retry path, or we serve the full body and let
      the client discard. v1 implements the former (simpler).
3. Blob-level success is not audited (≈20× noise per pull); the
   manifest-level `OciPullCompleted` is the canonical "pull happened"
   signal. Blob-level failures emit `OciPullFailedUpstream`.

### Tag change / cache invalidation

When an admin updates `pinned_image_tag` (or any `oci_image_*` field):

1. `ManifestCache` entries for `(app_id, *)` are invalidated.
2. An async task walks all active applications' current manifests
   (from cache if present, else re-fetched), collects the union of
   referenced blob digests, and unlinks `oci_blob_cache` rows whose
   digest is no longer referenced. Failure is logged but non-fatal;
   steady-state LRU eviction would also eventually reclaim this space.

A manual refresh endpoint mirrors the download proxy:
`POST /v1/admin/applications/<slug>/oci/refresh` re-fetches the pinned
tag's manifest and invalidates caches on demand. This lives on the
primary API (port `18080`), not the registry server.

## Components

New modules in `a8n-api`:

| File | Purpose |
| --- | --- |
| `api/src/services/forgejo_registry.rs` | HTTP client for Forgejo's `/v2/` endpoints. Validates upstream URL host/port/scheme against `FORGEJO_BASE_URL` before forwarding the API token. Passes client `Accept` headers through. Surfaces `Content-Type` and `Docker-Content-Digest`. |
| `api/src/services/manifest_cache.rs` | moka TTL cache over `(app_id, reference) → (bytes, media_type, digest)`. Invalidated on tag/field change. |
| `api/src/services/blob_cache.rs` | Disk cache keyed by digest. Mirrors `download_cache.rs`: SHA-256-named files, single-flight `Arc<OnceCell>`, atomic rename + fsync, async LRU eviction. Separate directory and size cap from release downloads. |
| `api/src/services/oci_token.rs` | Issues registry JWTs. Reuses platform JWT keypair with `aud: "registry"`, 15-min expiry. |
| `api/src/services/oci_limiter.rs` | Per-user manifest-concurrency slot + daily counter via `oci_pull_daily_counts`. RAII `OciPullGuard`. |
| `api/src/repositories/oci_blob_cache.rs` | `oci_blob_cache` table: `(digest PK, size_bytes, last_access_at, created_at)`. Used by the LRU evictor. |
| `api/src/repositories/oci_pull_daily_counts.rs` | `oci_pull_daily_counts` table, PK `(user_id, day_utc)`. |
| `api/src/middleware/oci_auth.rs` | Actix extractor `OciBearerUser`. Validates JWT, enforces `aud == "registry"`, re-checks active user + membership. Emits `401` with `WWW-Authenticate` on failure. |
| `api/src/errors/oci.rs` | `OciError` enum + `ResponseError` impl producing the OCI error envelope. |
| `api/src/handlers/oci_registry.rs` | All `/v2/*` handlers. |
| `api/src/handlers/oci_auth.rs` | `GET /auth/token` handler. |
| `api/src/routes/oci.rs` | Route registration + the secondary `HttpServer` factory. |

### Database migrations

- `*_add_oci_columns_to_applications.sql` — adds nullable
  `oci_image_owner TEXT`, `oci_image_name TEXT`, `pinned_image_tag TEXT`.
- `*_create_oci_blob_cache.sql` — disk-cache bookkeeping.
- `*_create_oci_pull_daily_counts.sql` — per-user daily counter.

### Audit actions (`audit_logs.action`)

- `OciLoginSucceeded` — token issued.
- `OciLoginFailed` — basic-auth rejected. Reason in `metadata`.
- `OciPullRequested` — limiter acquire succeeded; manifest fetch starting.
- `OciPullCompleted` — manifest delivered successfully.
- `OciPullFailedUpstream` — Forgejo error, digest mismatch, or mid-stream I/O failure (on manifest or blob).
- `OciPullDeniedRateLimit` — concurrency or daily cap denial.

### Admin UI

The `/admin/applications` edit form gains a new section matching the
existing Forgejo section pattern:

- **OCI Image Owner** — text field.
- **OCI Image Name** — text field.
- **Pinned Image Tag** — text field.

All three must be set for the application to be pullable. Changing
`pinned_image_tag` triggers manifest cache invalidation and the async
orphan-blob sweep.

## Error handling

Errors use the OCI spec error envelope:

```json
{"errors":[{"code":"MANIFEST_UNKNOWN","message":"...","detail":{...}}]}
```

| Condition | HTTP | OCI code |
| --- | --- | --- |
| Missing/invalid/expired JWT | 401 | `UNAUTHORIZED` (+ `WWW-Authenticate`) |
| JWT scope doesn't match requested `<slug>` | 403 | `DENIED` |
| User inactive / membership lapsed at request time | 401 | `UNAUTHORIZED` |
| App not found, inactive, or `oci_image_*` unset | 404 | `NAME_UNKNOWN` |
| Tag ≠ `pinned_image_tag` | 404 | `MANIFEST_UNKNOWN` |
| Blob digest not referenced by current manifest chain | 404 | `BLOB_UNKNOWN` |
| Concurrency limit hit | 429 | `TOOMANYREQUESTS` |
| Daily cap hit | 429 | `TOOMANYREQUESTS` (+ `Retry-After`) |
| Upstream 5xx / network failure / digest mismatch | 502 | `UNKNOWN` (no upstream details exposed) |
| Push verb on `/v2/*` | 405 | `UNSUPPORTED` |

Notes:

- **Blob digest mismatch** is treated as upstream failure: `.partial`
  deleted, no `oci_blob_cache` row inserted, `OciPullFailedUpstream`
  audited. Prevents cache poisoning.
- **Mid-stream client abort after clean upstream EOF** is not an error;
  the blob is fully cached and the pull can complete on retry.
- **Pinned-tag change mid-pull**: manifest cache is invalidated
  immediately; in-flight blob streams (keyed by digest) complete
  normally. The next manifest fetch picks up the new tag.

## Config

All loaded via `Config::from_env()`.

| Variable | Default | Purpose |
| --- | --- | --- |
| `OCI_REGISTRY_ENABLED` | `false` | Master switch. Registry server is only bound when `true`. |
| `OCI_REGISTRY_PORT` | `18081` | Internal port for the registry `HttpServer`. |
| `OCI_REGISTRY_SERVICE` | `registry.example.com` | `service` claim advertised in `WWW-Authenticate`. |
| `OCI_BLOB_CACHE_DIR` | `/var/cache/a8n-oci` | Directory for SHA-256-named blobs. |
| `OCI_BLOB_CACHE_MAX_BYTES` | `53687091200` (50 GiB) | Soft cap; async LRU eviction runs after each successful fetch when exceeded. |
| `OCI_MANIFEST_CACHE_TTL_SECS` | `300` | moka TTL for manifest metadata. |
| `OCI_CONCURRENT_MANIFESTS_PER_USER` | `2` | Simultaneous in-flight manifest fetches per user. |
| `OCI_PULLS_PER_USER_PER_DAY` | `50` | Manifest pulls per UTC day per user. |
| `OCI_TOKEN_TTL_SECS` | `900` | Registry bearer JWT lifetime. |

Docker Compose changes (`compose.yml`, `compose.dev.yml`):

- New named volume `oci_cache` → `/var/cache/a8n-oci` (dev:
  `saas-oci-cache-${USER}`).
- All eight env vars plumbed through to the `api` service.
- Registry port (`18081`) exposed on the `api` service for the reverse
  proxy / dev use.

## Testing

### Unit (`cargo test`)

- `services/forgejo_registry.rs`: URL host/port/scheme validation;
  Accept header pass-through; digest extraction.
- `services/manifest_cache.rs`: TTL expiry; invalidate-on-tag-change
  clears the right keys.
- `services/blob_cache.rs`: single-flight (two concurrent fetches →
  one upstream call); digest-mismatch rejection leaves no `.partial`
  and no row; LRU eviction under size pressure; atomic rename.
- `services/oci_token.rs`: `aud=registry` enforced on verify;
  API-audience token rejected; expired token rejected; scope parsing.
- `services/oci_limiter.rs`: concurrency slot released on drop; daily
  counter rolls at UTC midnight; denial rolls back the increment.
- `errors/oci.rs`: envelope shape matches OCI spec for each code.
- `handlers/oci_auth.rs`: basic-auth success; wrong password; inactive
  user; lapsed membership; scope targeting an app without
  `oci_image_*` set.

### Integration (`DATABASE_URL=… cargo test -- oci`)

Using `wiremock` to stub Forgejo's `/v2/` endpoints, mirroring the
download-proxy integration test layout:

- **Happy path pull.** Login → manifest → two blob fetches. Verifies
  `WWW-Authenticate` on the 401, token round trip, cache population,
  audit ordering (`OciLoginSucceeded` → `OciPullRequested` →
  `OciPullCompleted`).
- **Multi-arch (manifest index).** Upstream returns an OCI image
  index; client requests it with the right `Accept`; child manifest
  then fetched by digest; both cached.
- **Cache hit reduces upstream calls.** Second pull from a different
  user hits manifest + blob caches; wiremock call counts asserted.
- **Tag change invalidates.** Admin updates `pinned_image_tag`; next
  manifest fetch goes upstream; old-tag manifest cache is gone.
- **Scope mismatch.** Token for slug A used on slug B → `403 DENIED`.
- **Cross-audience token rejected.** Valid API JWT used as a registry
  bearer → `401`.
- **Rate limit denial.** Exceed `OCI_CONCURRENT_MANIFESTS_PER_USER` →
  `429` + `OciPullDeniedRateLimit`.
- **Digest mismatch upstream.** Wiremock returns bytes that don't hash
  to the requested digest → `502`, no cache insert, `.partial` gone.
- **Push verb blocked.** `POST /v2/<slug>/blobs/uploads/` → `405
  UNSUPPORTED`.
- **Feature flag off.** With `OCI_REGISTRY_ENABLED=false`, port 18081
  isn't bound; main API port still works.

### Manual smoke (`docs/oci-registry.md`)

1. Start dev compose with `OCI_REGISTRY_ENABLED=true` and real Forgejo
   credentials.
2. Seed an application with `oci_image_*` fields pointing at a real
   image in your Forgejo instance.
3. `docker login registry.example.localhost:18081` with a seeded
   member account.
4. `docker pull registry.example.localhost:18081/<slug>:<tag>`.
5. Verify audit rows and on-disk blobs in `/var/cache/a8n-oci`.

## Caveats / open items

- **Single-instance only** — `OciLimiter.inflight` is in-process. Multi-
  instance deploys would need a Postgres-backed replacement, same as
  the download proxy.
- **No PATs in v1** — email+password is the only credential; when
  a password rotates, `docker login` must rerun. PATs tracked as a
  follow-up.
- **Range requests on cold cache** are not honored natively; the first
  request populates the full blob, the client's retry then gets the
  range. Acceptable for `docker pull` (which doesn't use Range) but
  worth noting for other OCI clients.
- **Audit timing** mirrors the download proxy: `OciPullRequested` fires
  on acquire; `OciPullCompleted` fires when the manifest response
  completes, not when all blobs land. A pull that stalls mid-blob is
  a completed manifest pull with blob-level `OciPullFailedUpstream`.
- **Orphan blob sweep** on tag change is best-effort. LRU eviction is
  the durable backstop.

## Related

- `docs/forgejo-download-proxy.md` — release asset proxy; this feature
  mirrors many of its patterns (single-flight cache, LRU eviction,
  Forgejo URL validation, audit event structure).
- `docs/superpowers/specs/2026-04-15-forgejo-download-proxy-design.md`
  and `docs/superpowers/plans/2026-04-15-forgejo-download-proxy.md` —
  original design + implementation plan for the download proxy.
- OCI Distribution Spec v1.1 — https://github.com/opencontainers/distribution-spec/blob/main/spec.md
