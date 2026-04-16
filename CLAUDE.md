# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> For the full technical specification, see `a8n-tools-specification.md`.

## Project Overview

**example.com** is a SaaS platform hosting developer/productivity tools (URL shortener, bookmark manager). Rust API backend + React SPA frontend, with JWT-based SSO across subdomains (`*.example.com`).

## Development Commands

```bash
# Start development environment
docker compose -f compose.dev.yml up -d

# Run database migrations
cd api && cargo sqlx migrate run

# Stop / view logs / clean up
just down
just logs              # all services
just logs-api          # API only
just logs-frontend     # frontend only
just clean             # stop + remove volumes

# Database
just db-shell                           # psql into a8n_platform
just migrate                            # run migrations (cd api && cargo sqlx migrate run)
just migrate-create add_feature         # create new migration file

# Testing
just test              # all tests
just test-api          # cd api && cargo test
just test-frontend     # cd frontend && bun test (vitest watch mode)

# Deploy
docker compose pull && docker compose up -d

# View logs
docker compose logs -f api
```

# Frontend CI mode (no watch)
cd frontend && bun run test:run

```
a8n-tools/
├── api/                    # Rust backend (Actix-Web)
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/         # Route handlers
│   │   ├── models/         # Database models
│   │   ├── services/       # Business logic
│   │   ├── middleware/     # Auth, rate limiting
│   │   └── utils/          # Helpers
│   ├── migrations/         # SQL migrations
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/               # React SPA
│   ├── src/
│   ├── package.json
│   └── Dockerfile
├── apps/
│   ├── shared/
│   │   └── a8n-auth/      # Shared JWT auth library for child apps
│   ├── rus/               # RUS application
│   └── rustylinks/        # Rusty Links application
├── monitoring/
│   ├── prometheus.yml
│   └── grafana/
├── secrets/
│   ├── jwt_private.pem
│   └── jwt_public.pem
├── docker-compose.yml
├── docker-compose.dev.yml
├── .env.example
├── CLAUDE.md              # This file
└── a8n-tools-specification.md  # Full spec
```

# Linting
cd api && cargo clippy
cd api && cargo fmt
cd frontend && bun run lint

# Build Docker images
just build             # all
just build-api
just build-frontend
```

## Architecture

### Backend (api/)

Rust + Actix-Web. The crate is `a8n-api` (see `api/src/lib.rs` for module exports).

**Layered architecture:**
- `routes/` — Route registration (`.configure()` functions called from `routes::configure`)
- `handlers/` — HTTP request handlers (extract request data, call services/repos, return responses)
- `services/` — Business logic (auth, JWT, email, Stripe, password hashing)
- `repositories/` — Database access via sqlx (raw SQL, no ORM)
- `models/` — Data structures / DB models
- `middleware/` — Auth extractors, security headers, request ID
- `validation/` — Input validation (email, password strength, slugs)

**Request flow:** Route → Handler → Service/Repository → Response

**Key patterns:**
- Services are initialized in `main.rs` and injected via `web::Data<Arc<T>>` / `app_data()`
- Auth uses Actix extractors (`FromRequest` trait): `AuthenticatedUser`, `AdminUser`, `MemberUser`, `OptionalUser` — just add them as handler parameters
- All responses use `responses::success()`, `responses::created()`, `responses::paginated()` helpers that wrap data in `ApiResponse<T>` with `{ success, data, meta }` shape
- Errors use `AppError` enum (in `errors.rs`) which implements `ResponseError` — returns structured JSON with error code, message, and request ID
- `sqlx::Error` auto-converts to `AppError` (unique constraint → Conflict, row not found → NotFound)
- Config loaded from env vars via `Config::from_env()` (uses `dotenvy`)
- Migrations run automatically on startup (`sqlx::migrate!("./migrations")`)

### Frontend (frontend/)

React 18 + TypeScript + Vite + Tailwind CSS + shadcn/ui.

**Key patterns:**
- Path alias: `@/` maps to `src/` (configured in vite.config.ts and tsconfig.json)
- API calls go through `apiClient` (`api/client.ts`) which uses `fetch` with `credentials: 'include'`
- Vite dev server proxies `/api` → API server, rewriting to `/v1` (see `vite.config.ts`)
- State management: Zustand stores (`stores/authStore.ts` persists auth state to localStorage)
- Route protection: `ProtectedRoute` and `AdminRoute` wrapper components in `App.tsx`
- Tests use Vitest + Testing Library + MSW for API mocking (`src/test/` has setup, utils, mock handlers)
- Custom `render` from `src/test/utils.tsx` wraps components with QueryClient + BrowserRouter providers
- UI components from shadcn/ui in `components/ui/`

### Auth & SSO

- JWT tokens stored in HTTP-only cookies, domain set via `COOKIE_DOMAIN` env var (e.g. `.example.com`)
- Access token: 15 min, Refresh token: 30 days
- Cookie set/cleared via `AuthCookies` helper in `middleware/auth.rs`
- Token extracted from `access_token` cookie first, then `Authorization: Bearer` header
- Child apps (RUS, Rusty Links) validate JWT locally using shared `JWT_SECRET`

### Database

PostgreSQL 16. Migrations in `api/migrations/` (sqlx, sequential numbering `20241230000001_*`). Platform DB holds users, tokens, memberships, audit logs. Each child app has its own isolated DB.

## CI/CD

Forgejo Actions (`.forgejo/workflows/`). On push to `main`, builds OCI images using `docker buildx build` with the project Dockerfiles and pushes to Forgejo Container Registry. Tag resolution uses `oci-build/get-tags.nu`.
1. **Always use parameterized queries** — sqlx handles this automatically
2. **Hash tokens before storing** — never store raw magic link or reset tokens
3. **Log all auth events** — audit_logs table for security tracking
4. **Test Stripe webhooks locally** — use Stripe CLI: `stripe listen --forward-to localhost:4000/v1/webhooks/stripe`
5. **JWT public key shared with apps** — mount as read-only volume
6. **Cookie domain is set via `COOKIE_DOMAIN` env var** — enables SSO across subdomains
7. **Validate membership status on every app request** — check JWT claims
8. **Admin actions require extra logging** — set `is_admin_action = true` in audit logs
9. **Encryption keys for TOTP and Stripe** — generate with `openssl rand -hex 32`, see [`docs/encryption-key-rotation.md`](docs/encryption-key-rotation.md) for rotation workflow

## Conventions

- **Commits:** Conventional Commits format — `type(scope): description` (feat, fix, docs, refactor, test, etc.)
- **API routes:** All under `/v1` scope. Add new routes by creating handler in `handlers/`, route config in `routes/`, and registering in `routes/mod.rs`
- **Frontend pages:** Add page in `pages/`, register route in `App.tsx`, update navigation if needed
- **Password rules:** 12+ chars, mixed case, digit, special character (see `validation/mod.rs`)
- **Color theme:** Primary orange `#f97316`, Rust accent `#b7410e`

## Dev Environment URLs

- Frontend: http://localhost:5173
- API: http://localhost:18080

## Feature Flags

### Download proxy
Gated behind `FORGEJO_BASE_URL` + `FORGEJO_API_TOKEN`. Downloads stream from
Forgejo through the API to logged-in members with active membership. Files are
cached on disk at `DOWNLOAD_CACHE_DIR` (defaults to the named volume
`a8n-tools-downloads`). See `docs/forgejo-download-proxy.md` for user + dev
documentation (config vars, audit actions, caveats). Original design +
implementation plan: `docs/superpowers/specs/2026-04-15-forgejo-download-proxy-design.md`
and `docs/superpowers/plans/2026-04-15-forgejo-download-proxy.md`.

### OCI registry
Gated behind `OCI_REGISTRY_ENABLED=true` + `FORGEJO_BASE_URL` + `FORGEJO_API_TOKEN`.
Second HTTP server at `OCI_REGISTRY_PORT` (default 18081) exposes an OCI-compliant
read-only registry. Members `docker login` with their a8n credentials and
`docker pull <registry>/<app-slug>:<pinned-tag>`. Blobs cached on disk at
`OCI_BLOB_CACHE_DIR` (volume `oci_cache`). See `docs/oci-registry.md` for user +
dev documentation. Original design + implementation plan:
`docs/superpowers/specs/2026-04-16-oci-registry-design.md` and
`docs/superpowers/plans/2026-04-16-oci-registry.md`.
