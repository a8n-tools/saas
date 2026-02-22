# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> For the full technical specification, see `a8n-tools-specification.md`.

## Project Overview

**example.com** is a SaaS platform hosting developer/productivity tools (URL shortener, bookmark manager). Rust API backend + React SPA frontend, with JWT-based SSO across subdomains (`*.example.com`).

## Development Commands

```bash
# Start full dev environment (Postgres + API + Frontend via Docker Compose)
just dev

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

# Run a single Rust test
cd api && cargo test test_name

# Run a single frontend test file
cd frontend && bunx vitest run src/path/to/file.test.ts

# Frontend CI mode (no watch)
cd frontend && bun run test:run

# Frontend coverage
cd frontend && bun run test:coverage

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

- JWT tokens stored in HTTP-only cookies on `.example.com` domain (`.a8n.run` in dev)
- Access token: 15 min, Refresh token: 30 days
- Cookie set/cleared via `AuthCookies` helper in `middleware/auth.rs`
- Token extracted from `access_token` cookie first, then `Authorization: Bearer` header
- Child apps (RUS, Rusty Links) validate JWT locally using shared `JWT_SECRET`

### Database

PostgreSQL 16. Migrations in `api/migrations/` (sqlx, sequential numbering `20241230000001_*`). Platform DB holds users, tokens, memberships, audit logs. Each child app has its own isolated DB.

## CI/CD

Forgejo Actions (`.forgejo/workflows/`). On push to `main`, builds OCI images using `docker buildx build` with the project Dockerfiles and pushes to Forgejo Container Registry. Tag resolution uses `oci-build/get-tags.nu`.

## Conventions

- **Commits:** Conventional Commits format — `type(scope): description` (feat, fix, docs, refactor, test, etc.)
- **API routes:** All under `/v1` scope. Add new routes by creating handler in `handlers/`, route config in `routes/`, and registering in `routes/mod.rs`
- **Frontend pages:** Add page in `pages/`, register route in `App.tsx`, update navigation if needed
- **Password rules:** 12+ chars, mixed case, digit, special character (see `validation/mod.rs`)
- **Color theme:** Primary orange `#f97316`, Rust accent `#b7410e`

## Dev Environment URLs

- Frontend: http://localhost:5173
- API: http://localhost:18080
