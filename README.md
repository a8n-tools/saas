# a8n.tools

A SaaS platform hosting developer and productivity tools. We sell convenience, reliability, and managed hosting for
open-source applications.

## Overview

**a8n.tools** provides hosted versions of open-source developer tools with:

- No server setup, maintenance, or updates required
- Managed infrastructure with monitoring and backups
- Dedicated support for subscribers
- **$3/month** flat price for access to all current and future tools
- **Fixed price for life** - early adopters lock in their rate forever

### Current Applications

| Application | Description                  | Subdomain              |
|-------------|------------------------------|------------------------|
| RUS         | URL shortening with QR codes | `rus.a8n.tools`        |
| Rusty Links | Bookmark management          | `rustylinks.a8n.tools` |

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
    - API: http://localhost:8080
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
├── docker-compose.dev.yml  # Development environment
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

### Navigate to frontend directory

cd frontend

#### Run tests in watch mode (re-runs on file changes)

bun test

#### Run tests once (CI mode)

bun run test:run

#### Run tests with coverage report

bun run test:coverage

## Current Test Coverage

auth.test.ts - 13 tests (login, register, logout, magic link, password reset)
authStore.test.ts - 17 tests (state management, login/logout flow, error handling)

## Check if migrations are in sync

Run this command if the _sqlx_migrations table was emptied on accident
If this returns 0 but tables exist, you know there's a problem before the API crashes.

```
docker exec a8n-tools-postgres psql -U a8n -d a8n_platform -c \
   "SELECT COUNT(*) FROM _sqlx_migrations;"
```

## Admin Setup

To promote a user to admin, connect to the database and update their role:

```bash
just db-shell
```

```sql
UPDATE users
SET role = 'admin'
WHERE email = 'your@email.com';
```

Once you have an admin account, you can promote additional users from the admin UI at the Users page.

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
// api/src/handlers/users.rs
use actix_web::{get, web, HttpResponse};
use crate::responses::success;

#[get("/users/me")]
async fn get_current_user() -> HttpResponse {
    // Handler implementation
}
```

### Adding a New Frontend Page

1. Create the page component in `frontend/src/pages/`
2. Add the route in `frontend/src/App.tsx`
3. Update navigation if needed

## Environment Variables

| Variable                | Description                      | Default                 | Required   |
|-------------------------|----------------------------------|-------------------------|------------|
| `DATABASE_URL`          | PostgreSQL connection string     | -                       | Yes        |
| `HOST`                  | API server host                  | `0.0.0.0`               | No         |
| `PORT`                  | API server port                  | `8080`                  | No         |
| `RUST_LOG`              | Log level                        | `info`                  | No         |
| `CORS_ORIGIN`           | Allowed CORS origin              | `https://app.a8n.tools` | No         |
| `ENVIRONMENT`           | Environment name                 | `development`           | No         |
| `JWT_PRIVATE_KEY_PATH`  | Path to Ed25519 private key      | -                       | Yes (prod) |
| `JWT_PUBLIC_KEY_PATH`   | Path to Ed25519 public key       | -                       | Yes (prod) |
| `STRIPE_SECRET_KEY`     | Stripe API secret key            | -                       | Yes (prod) |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret    | -                       | Yes (prod) |
| `STRIPE_PRICE_ID`       | Stripe price ID for subscription | -                       | Yes (prod) |

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
- **Cookie Domain**: `.a8n.tools` - enables SSO across subdomains

### Subdomain Routing

Traefik handles routing based on subdomain:

- `a8n.tools` -> Marketing site
- `app.a8n.tools` -> User dashboard
- `api.a8n.tools` -> Backend API
- `admin.a8n.tools` -> Admin panel
- `*.a8n.tools` -> Individual applications

Will this work on any machine?

Almost — the only manual step is each developer needs to add the /etc/hosts entries:

- 127.0.0.1 a8n.test
- 127.0.0.1 app.a8n.test
- 127.0.0.1 api.a8n.test
- 127.0.0.1 rus.a8n.test

Everything else (Traefik routing, cookie domain, CORS) is baked into the compose files and code. So for any new dev
machine it's: clone, add hosts entries, docker
compose up.

You could automate the hosts step with a Makefile target or a setup script if you wanted to reduce friction for the
other two devs.

The Firefox proxy method

That approach involves configuring Firefox (or a PAC file) to route *.a8n.test traffic through a local proxy. It avoids
touching /etc/hosts but adds complexity. With
your current setup — /etc/hosts + Traefik — you get the same result more simply. No need for it.

What changes for production?

Almost nothing — your production docker-compose.yml is already set up correctly:

| Concern            | Dev (current)                | Production (already handled)                      |
|--------------------|------------------------------|---------------------------------------------------|
| DNS                | /etc/hosts → 127.0.0.1       | Real DNS records for *.a8n.tools                  |
| TLS                | None (HTTP)                  | Let's Encrypt via Traefik (already configured)    |
| Cookie domain      | .a8n.test (explicit env var) | .a8n.tools (auto-set when ENVIRONMENT=production) |
| Cookie Secure flag | false                        | true (from config.is_production())                |
| CORS               | .a8n.test + .a8n.tools       | .a8n.tools (already in code)                      |
| Vite allowedHosts  | Needed for dev server        | N/A — production serves static files via nginx    |

The only thing to confirm is that each child app in production shares the same JWT_SECRET env var as the main API. Your
production compose already has JWT_SECRET:
${JWT_SECRET} on the API — just make sure RUS and any other child apps get the same value.

## License

Proprietary - All Rights Reserved

- https://en.wikipedia.org/wiki/Business_models_for_open-source_software
