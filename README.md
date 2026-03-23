# a8n.tools

A SaaS platform hosting developer and productivity tools. We sell convenience, reliability, and managed hosting for open-source applications.

## Overview

**a8n.tools** provides hosted versions of open-source developer tools with:
- No server setup, maintenance, or updates required
- Managed infrastructure with monitoring and backups
- Dedicated support for subscribers
- **$3/month** flat price for access to all current and future tools
- **Fixed price for life** - early adopters lock in their rate forever

### Current Applications

| Application  | Description                  | Subdomain              |
|--------------|------------------------------|------------------------|
| RUS          | URL shortening with QR codes | `rus.a8n.tools`        |
| Rusty Links  | Bookmark management          | `rustylinks.a8n.tools` |

## Tech Stack

| Layer            | Technology                                           |
|------------------|------------------------------------------------------|
| Backend          | Rust, Actix-Web                                      |
| Frontend         | React 18+, Vite, TypeScript, Tailwind CSS, shadcn/ui |
| Database         | PostgreSQL 16+                                       |
| Containerization | Docker, Docker Compose                               |
| Reverse Proxy    | Traefik                                              |
| Email            | Stalwart (self-hosted)                               |
| Monitoring       | Prometheus, Grafana                                  |
| Error Tracking   | GlitchTip (self-hosted)                              |

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust toolchain (for local development)
- Bun
- Git

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/a8n-tools.git
   cd a8n-tools
   ```

2. Copy environment file:
   ```bash
   cp .env.example .env
   ```

3. Start the development environment:
   ```bash
   just dev
   ```

4. Access the applications:
   - Frontend: http://localhost:5173
   - API: http://localhost:4000
   - Traefik Dashboard: http://localhost:8081

   With Traefik routing:
   - Frontend: http://localhost
   - API: http://api.localhost

5. Add to `/etc/hosts` (optional, for subdomain routing):
   ```
   127.0.0.1 localhost api.localhost admin.localhost
   ```

## Project Structure

```
a8n-tools/
├── api/                    # Rust backend (Actix-Web)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── config.rs       # Configuration loading
│   │   ├── errors.rs       # Error types
│   │   ├── responses.rs    # Response types
│   │   ├── routes/         # Route definitions
│   │   ├── handlers/       # Request handlers
│   │   ├── models/         # Data models
│   │   ├── services/       # Business logic
│   │   └── middleware/     # Custom middleware
│   ├── migrations/         # Database migrations
│   ├── Cargo.toml
│   └── Dockerfile.dev
├── frontend/               # React frontend
│   ├── src/
│   │   ├── api/            # API client
│   │   ├── components/     # UI components
│   │   ├── pages/          # Page components
│   │   ├── hooks/          # Custom hooks
│   │   ├── stores/         # Zustand stores
│   │   ├── lib/            # Utilities
│   │   └── types/          # TypeScript types
│   ├── package.json
│   └── Dockerfile.dev
├── apps/                   # Hosted applications
│   ├── rus/
│   └── rustylinks/
├── traefik/                # Traefik configuration
├── docs/                   # Documentation
├── compose.dev.yml         # Development environment
├── Justfile                # Development commands
└── .env.example            # Environment template
```

## Test Structure

```
frontend/src/
  ├── test/
  │   ├── setup.ts          # Test setup (jest-dom, MSW server)
  │   ├── utils.tsx          # Custom render with providers
  │   └── mocks/
  │       ├── handlers.ts    # MSW API mock handlers
  │       └── server.ts      # MSW server instance
  ├── api/
  │   └── auth.test.ts       # Auth API tests
  └── stores/
      └── authStore.test.ts  # Auth store tests
```

```bash
cd frontend

# Run tests in watch mode (re-runs on file changes)
bun test

# Run tests once (CI mode)
bun run test:run

# Run tests with coverage report
bun run test:coverage
```

## Check if migrations are in sync

Run this command if the _sqlx_migrations table was emptied on accident
If this returns 0 but tables exist, you know there's a problem before the API crashes.

```
docker exec a8n-postgres psql -U a8n -d a8n_platform -c \
   "SELECT COUNT(*) FROM _sqlx_migrations;"
```

## Development

### Available Commands

Run `just --list` to see all available commands:

```bash
# Start development environment
just dev

# Stop all services
just down

# View logs
just logs
just logs-api
just logs-frontend

# Database operations
just db-shell       # Connect to PostgreSQL
just migrate        # Run migrations
just migrate-create create_users  # Create new migration

# Testing
just test           # Run all tests
just test-api       # Run API tests only
just test-frontend  # Run frontend tests only

# Build
just build          # Build all Docker images

# Cleanup
just clean          # Stop services and remove volumes
```

### Adding a New API Endpoint

1. Create a handler in `api/src/handlers/`
2. Define the route in `api/src/routes/`
3. Register the route in `api/src/routes/mod.rs`

Example:

```rust
// api/src/handlers/example.rs
use actix_web::{web, HttpRequest, HttpResponse};
use crate::errors::AppError;
use crate::responses::{get_request_id, success};

pub async fn get_item(
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    Ok(success(serde_json::json!({ "item": "value" }), request_id))
}
```

### Adding a New Frontend Page

1. Create the page component in `frontend/src/pages/`
2. Add the route in `frontend/src/App.tsx`
3. Update navigation if needed

## Environment Variables

### API (Backend)

| Variable                | Description                                                  | Default                                                   | Required   |
|-------------------------|--------------------------------------------------------------|-----------------------------------------------------------|------------|
| `DATABASE_URL`          | PostgreSQL connection string                                 | -                                                         | Yes        |
| `HOST_IP`               | API server host                                              | `0.0.0.0`                                                 | No         |
| `APP_PORT`              | API server port                                              | `8080`                                                    | No         |
| `RUST_LOG`              | Log level                                                    | `info`                                                    | No         |
| `ENVIRONMENT`           | `production` or `development`                                | `production`                                              | No         |
| `APP_NAME`              | App name used in email subjects and templates                | `localhost`                                               | No         |
| `APP_URL`               | Frontend base URL for email links                            | Falls back to `CORS_ORIGIN`, then `http://localhost:5173` | No |
| `CORS_ORIGIN`           | Allowed CORS origin (frontend URL)                           | `http://localhost:5173`                                   | No         |
| `COOKIE_DOMAIN`         | Cookie domain for cross-subdomain auth (e.g. `.example.com`) | None (exact hostname)                                     | Yes (prod) |
| `JWT_SECRET`            | Shared JWT signing secret                                    | -                                                         | Yes (prod) |
| `TOTP_ENCRYPTION_KEY`   | Hex-encoded 32-byte key for encrypting TOTP secrets          | Zero bytes (dev only)                                     | Yes (prod) |
| `STRIPE_ENCRYPTION_KEY` | Hex-encoded 32-byte key for encrypting Stripe secrets        | Zero bytes (dev only)                                     | Yes (prod) |
| `STRIPE_SECRET_KEY`     | Stripe API secret key                                        | -                                                         | Yes (prod) |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret                                | -                                                         | Yes (prod) |
| `STRIPE_PRICE_ID`       | Stripe price ID for subscription                             | -                                                         | Yes (prod) |
| `SMTP_HOST`             | SMTP server hostname                                         | `localhost`                                               | No         |
| `SMTP_PORT`             | SMTP server port                                             | `465`                                                     | No         |
| `SMTP_FROM`             | Sender email (format: `Name <email>` or `email`)             | `noreply@localhost`                                       | No         |
| `SMTP_USERNAME`         | SMTP auth username                                           | -                                                         | No         |
| `SMTP_PASSWORD`         | SMTP auth password                                           | -                                                         | No         |
| `EMAIL_ENABLED`         | Force enable email sending in dev                            | `false`                                                   | No         |

### Frontend

| Variable                    | Description                              | Default                   | Required   |
|-----------------------------|------------------------------------------|---------------------------|------------|
| `VITE_API_URL`              | API base URL                             | `http://localhost:18080`  | Yes (prod) |
| `VITE_APP_DOMAIN`           | Application domain                       | `localhost`               | No         |
| `VITE_SHOW_BUSINESS_PRICING`| Show business pricing tier               | `false`                   | No         |

Frontend env vars are injected at runtime via a Caddy template endpoint (obfuscated path), not baked into the build. This allows deploying the same image to different environments by changing container env vars.

## Health Checks

Both the API and frontend images expose a `/health` endpoint but do **not** include a
built-in `HEALTHCHECK` instruction. It is the deployer's responsibility to configure
health checks in their compose file or orchestrator.

| Service    | Endpoint  | Port | Healthy response   |
|------------|-----------|------|--------------------|
| `api`      | `/health` | 8080 | `200 OK`           |
| `frontend` | `/health` | 8080 | `200 OK` "healthy" |

### Docker Compose example

```yaml
services:
  api:
    image: your-registry/saas-api:latest
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      start_period: 5s
      retries: 3

  frontend:
    image: your-registry/saas-frontend:latest
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      start_period: 5s
      retries: 3
```

## Architecture Decisions

### Why Actix-Web?

- Excellent performance and async support
- Strong ecosystem for web services
- Type-safe request handling
- Battle-tested in production

### Why SQLx over other ORMs?

- Compile-time query verification
- No runtime ORM overhead
- Direct SQL with type safety
- Async-first design

### JWT Strategy

- **Algorithm**: EdDSA (Ed25519) - faster and more secure than RS256
- **Access Token**: 15 minutes - short-lived for security
- **Refresh Token**: 30 days - stored in database for revocation
- **Cookie Domain**: `.example.com` - enables SSO across subdomains

### Subdomain Routing

Traefik handles routing based on subdomain:
- `a8n.tools` -> Marketing site
- `app.a8n.tools` -> User dashboard
- `api.a8n.tools` -> Backend API
- `admin.a8n.tools` -> Admin panel
- `*.a8n.tools` -> Individual applications


Will this work on any machine?                                                                                                                                        
                                                                                                                                                                        
  Almost — the only manual step is each developer needs to add the /etc/hosts entries:                                                                                  
   
  127.0.0.1 a8n.test                                                                                                                                                    
  127.0.0.1 app.a8n.test                                                           
  127.0.0.1 api.a8n.test
  127.0.0.1 rus.a8n.test

- `example.com` -> Marketing site
- `app.example.com` -> User dashboard
- `api.example.com` -> Backend API
- `admin.example.com` -> Admin panel
- `*.example.com` -> Individual applications

### Dev vs Production

| Concern            | Dev                          | Production                                        |
|--------------------|------------------------------|---------------------------------------------------|
| DNS                | `localhost`                  | Real DNS records for `*.example.com`              |
| TLS                | None (HTTP)                  | Let's Encrypt via Traefik                         |
| Cookie domain      | None (exact hostname)        | `.example.com` (set via `COOKIE_DOMAIN`)          |
| Cookie Secure flag | `false`                      | `true` (when `ENVIRONMENT=production`)            |
| CORS               | `http://localhost:5173`      | Frontend URL (set via `CORS_ORIGIN`)              |
| Frontend serving   | Vite dev server              | Static files via Caddy                            |
| Email links        | `http://localhost:5173`      | Derived from `APP_URL` or `CORS_ORIGIN`           |

Each child app (RUS, Rusty Links) must share the same `JWT_SECRET` as the main API for SSO to work.

## License

Proprietary - All Rights Reserved

- https://en.wikipedia.org/wiki/Business_models_for_open-source_software
