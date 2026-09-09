# Quickstart

Get a fresh clone of the a8n.tools platform running, and find the task-runner recipe for the thing you are trying to do.

## Prerequisites

- Docker with Compose v2
- [just](https://github.com/casey/just)
- [Nushell](https://www.nushell.sh/), for the `create-release` recipe only
- Git

A host Rust toolchain and Bun are optional. Every build, test, lint, and migration recipe runs inside the dev containers, so a host toolchain is only needed if you want to run `cargo` or `bun` outside them.

## Clone and start

```bash
git clone https://dev.a8n.run/a8n-tools/saas.git
cd saas
just dev
```

`just dev` depends on the private `ensure-env` recipe, which creates `.env`, `api/.env`, and `frontend/.env` from their `.example` files if they are missing. There is no separate copy step.

Use `just dev-detach` to start the same stack in the background.

## The two compose files

| File | What it is | How to start it |
|------|------------|-----------------|
| `compose.dev.yml` | The Traefik-routed dev stack. Every service joins the external `network-traefik-public` network and is reached over TLS at per-developer hostnames. | `just dev` |
| `compose.yml` | The plain localhost dev stack, with no Traefik and host-published ports. | `docker compose up --build` |
| `examples/compose.yml` | A reference deployment using the published `saas-api` and `saas-frontend` images. Not a dev stack. | copy and adapt |

### URLs under `compose.dev.yml`

Routing is by hostname, not by published port. The only port `compose.dev.yml` publishes to the host is `18081`, the optional OCI registry.

- Front end: `https://${USER}-app.a8n.run`
- API: `https://${USER}-api.a8n.run`

`just dev-sso` starts the stack and prints these two URLs; it exists so single sign-on can be tested against real DNS and real certificates, which localhost cannot provide.

Note that the `just dev-detach` recipe still echoes `http://localhost:5173` and `http://localhost:18080`. Those are the `compose.yml` addresses, not the `compose.dev.yml` ones, so ignore them when you started the stack with `just dev` or `just dev-detach`.

### URLs under `compose.yml`

- Front end: <http://localhost:5173>
- API: <http://localhost:18080>, which maps to the container's `APP_PORT` of `4000`

## Container and volume names

`compose.dev.yml` creates `dev-saas-postgres-${USER}`, `dev-saas-api-${USER}`, and `dev-saas-frontend-${USER}`, with volumes `saas-data-${USER}`, `saas-api-target-${USER}`, `saas-downloads-cache-${USER}`, and `saas-oci-cache-${USER}`.

`compose.yml` creates `dev-a8n-tools-postgres-${USER}`, `dev-a8n-tools-api-${USER}`, and `dev-a8n-tools-frontend-${USER}`.

## Recipes

`just --list` shows every recipe. The ones you will reach for:

```bash
# Lifecycle
just dev            # start (foreground), building images as needed
just dev-detach     # start in the background
just dev-sso        # start in the background and print the a8n.run hostnames
just down           # stop the stack
just clean          # stop, remove volumes, and drop the OCI cache volume

# Logs
just logs           # follow every service
just logs-api
just logs-frontend

# Database
just db-shell                       # psql into a8n_platform as the a8n role
just migrate                        # cargo sqlx migrate run, inside the api container
just migrate-create add_feature     # cargo sqlx migrate add, inside the api container

# Tests
just test                           # test-api then test-frontend
just test-api                       # cargo test --lib in the api container, with GIT_COMMIT=dev
just test-frontend                  # bun run test:run in the frontend container

# Lint and format
just lint-api                       # cargo clippy in the api container
just fmt-api                        # cargo fmt in the api container
just lint-frontend                  # bun run lint in the frontend container

# Images
just build                          # build both dev images
just build-api
just build-frontend
just check-docker-api               # build the oci-build API image, builder stage only
just check-docker-frontend          # build the oci-build frontend image
just build-docker-api               # tag saas-api:local
just build-docker-frontend          # tag saas-frontend:local
```

The `test-dev-api`, `test-dev-frontend`, and `test-dev` recipes call `docker exec` against containers named `saas-api-$USER` and `saas-frontend-$USER`. Neither compose file creates containers under those names, so use `just test-api` and `just test-frontend` instead.

## Frontend tests

Test scaffolding lives under `frontend/src/test/`:

```text
frontend/src/
  test/
    setup.ts            # Vitest setup: jest-dom matchers and the MSW server
    utils.tsx           # custom render wrapping components in QueryClient and BrowserRouter
    mocks/
      handlers.ts       # MSW API mock handlers
      server.ts         # MSW server instance
  api/auth.test.ts
  stores/authStore.test.ts
```

Outside the containers, from `frontend/`:

```bash
bun test              # watch mode
bun run test:run      # single run, the mode CI uses
bun run test:coverage # single run with a coverage report
```

## Repository layout

```text
api/                    # Rust backend (Actix Web)
  src/
    main.rs             # entry point
    lib.rs              # module exports; the crate is a8n-api
    config.rs           # Config::from_env, the authoritative env var reader
    errors/             # AppError and its ResponseError impl
    responses.rs        # ApiResponse helpers: success, created, paginated
    routes/             # route registration
    handlers/           # HTTP request handlers
    services/           # business logic: auth, JWT, email, Stripe
    repositories/       # sqlx database access, raw SQL
    models/             # data structures and DB models
    middleware/         # auth extractors, security headers, request id
    validation/         # email, password strength, slug validation
  migrations/           # sqlx migrations, sequential numbering
  Cargo.toml
  Dockerfile            # dev image
frontend/               # React SPA
  src/
    App.tsx             # route table, including ProtectedRoute and AdminRoute
    api/                # apiClient and per-domain API modules
    components/         # UI components, with shadcn/ui under components/ui
    pages/
    hooks/
    stores/             # Zustand stores
    lib/
    styles/
    types/
    test/               # Vitest setup, render helpers, and MSW handlers
  package.json
  Dockerfile            # dev image
oci-build/              # production Dockerfiles and the get-tags.nu tag resolver
examples/               # a reference deployment compose file
troubleshooting/        # a small standalone crate for reproducing build issues
docs/
compose.dev.yml         # Traefik-routed dev stack
compose.yml             # plain localhost dev stack
justfile                # task runner
.env.example
```

## Check whether migrations are in sync

If the `_sqlx_migrations` table was emptied by accident, this returns `0` while the tables still exist, which tells you before the API crashes on startup:

```bash
docker exec dev-saas-postgres-${USER} psql -U a8n -d a8n_platform -c \
  "SELECT COUNT(*) FROM _sqlx_migrations;"
```

Under `compose.yml` the container is `dev-a8n-tools-postgres-${USER}` instead.

## Adding a new API endpoint

1. Create a handler in `api/src/handlers/`.
2. Define the route in `api/src/routes/`.
3. Register the route in `api/src/routes/mod.rs`. Every route lives under the `/v1` scope.

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

## Adding a new frontend page

1. Create the page component in `frontend/src/pages/`.
2. Add the route in `frontend/src/App.tsx`.
3. Update navigation if needed.

## Releasing

Cutting a release is covered in [releasing.md](releasing.md).
