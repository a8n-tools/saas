# 01 - Project Setup & Development Environment

## Overview

This document contains prompts for setting up the foundational project structure, Docker development environment, and basic configuration.

## Prerequisites
- Docker and Docker Compose installed
- Rust toolchain (rustup)
- Bun
- Git

---

## Prompt 1.1: Create Rust API Project Scaffold

```text
Create a new Rust API project for example.com platform with Actix-Web.

Project requirements:
- Name: a8n-api
- Use Actix-Web (latest stable)
- Use SQLx for PostgreSQL with compile-time verification
- Use tokio as async runtime
- Structure the project with clear separation:
  - src/main.rs - Entry point
  - src/config.rs - Configuration loading
  - src/lib.rs - Library exports
  - src/errors.rs - Error types
  - src/routes/ - Route definitions
  - src/handlers/ - Request handlers
  - src/models/ - Data models
  - src/services/ - Business logic
  - src/middleware/ - Custom middleware

Create Cargo.toml with these dependencies:
- actix-web = "4"
- actix-cors = "0.7"
- tokio = { version = "1", features = ["full"] }
- sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
- serde = { version = "1", features = ["derive"] }
- serde_json = "1"
- chrono = { version = "0.4", features = ["serde"] }
- uuid = { version = "1", features = ["v4", "serde"] }
- dotenvy = "0.15"
- tracing = "0.1"
- tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
- thiserror = "1"
- anyhow = "1"

Create a basic main.rs that:
1. Loads configuration from environment variables
2. Sets up structured JSON logging with tracing
3. Creates a database connection pool
4. Configures CORS for .example.com domain
5. Starts the server on configurable port (default 8080)
6. Has a /health endpoint returning {"status": "ok"}

Create config.rs that loads:
- DATABASE_URL
- HOST (default: 0.0.0.0)
- PORT (default: 8080)
- RUST_LOG (default: info)
- CORS_ORIGIN (default: https://app.example.com)

Include proper error handling - don't use unwrap() in production code.

Create a .env.example file with all required variables.
```

---

## Prompt 1.2: Create Error Handling Foundation

```text
Create a comprehensive error handling system for the a8n-api.

In src/errors.rs, create:

1. An AppError enum with variants:
   - ValidationError { field: String, message: String }
   - InvalidCredentials
   - TokenExpired
   - Unauthorized
   - Forbidden
   - NotFound { resource: String }
   - Conflict { message: String }
   - RateLimited { retry_after: u64 }
   - InternalError { message: String }
   - DatabaseError { message: String }

2. Implement ResponseError for AppError that returns:
   - Appropriate HTTP status codes
   - JSON body in this format:
   ```json
   {
     "success": false,
     "error": {
       "code": "ERROR_CODE",
       "message": "Human-readable message",
       "details": {}
     },
     "meta": {
       "request_id": "req_xxx",
       "timestamp": "ISO8601"
     }
   }
   ```

3. Create error code mappings:
   - VALIDATION_ERROR -> 400
   - INVALID_CREDENTIALS -> 401
   - TOKEN_EXPIRED -> 401
   - UNAUTHORIZED -> 401
   - FORBIDDEN -> 403
   - NOT_FOUND -> 404
   - CONFLICT -> 409
   - RATE_LIMITED -> 429
   - INTERNAL_ERROR -> 500
   - DATABASE_ERROR -> 500

4. Implement From<sqlx::Error> for AppError

5. Create a helper macro or function for creating validation errors

6. Add request ID generation (use uuid v4 with "req_" prefix)

Write tests for:
- Each error variant produces correct status code
- Error JSON structure is correct
- Request IDs are properly generated
```

---

## Prompt 1.3: Create Standard API Response Types

```text
Create standardized API response types for a8n-api.

In src/responses.rs, create:

1. A generic ApiResponse<T> struct:
   ```rust
   pub struct ApiResponse<T> {
       pub success: bool,
       pub data: Option<T>,
       pub meta: ResponseMeta,
   }
   ```

2. ResponseMeta struct:
   ```rust
   pub struct ResponseMeta {
       pub request_id: String,
       pub timestamp: DateTime<Utc>,
   }
   ```

3. Helper functions:
   - `success<T>(data: T, request_id: String) -> HttpResponse`
   - `success_no_data(request_id: String) -> HttpResponse`
   - `created<T>(data: T, request_id: String) -> HttpResponse`

4. Pagination support:
   ```rust
   pub struct PaginatedResponse<T> {
       pub items: Vec<T>,
       pub total: i64,
       pub page: i32,
       pub per_page: i32,
       pub total_pages: i32,
   }
   ```

5. Create a middleware or extractor that:
   - Generates a request_id for each request
   - Stores it in request extensions
   - Makes it available to handlers

Ensure all responses:
- Are JSON
- Include the request_id
- Include ISO8601 timestamp
- Use consistent casing (snake_case)

Write unit tests for response serialization.
```

---

## Prompt 1.4: Create Docker Development Environment

```text
Create a Docker Compose development environment for example.com.

Create docker-compose.dev.yml with these services:

1. **postgres** (PostgreSQL 16):
   - Volume for data persistence
   - Health check
   - Port 5432 exposed for local development
   - Database: a8n_platform
   - User: a8n

2. **api** (Rust backend):
   - Build from ./api
   - Mount source code for live reload (use cargo-watch)
   - Depends on postgres
   - Environment variables from .env
   - Port 8080

3. **frontend** (React dev server):
   - Build context ./frontend
   - Mount source code for HMR
   - Port 5173
   - Environment: VITE_API_URL=http://localhost:8080

Create a Dockerfile.dev for the API that:
- Uses rust:1.75
- Installs cargo-watch
- Runs: cargo watch -x run

Create a Dockerfile.dev for frontend that:
- Uses node:20-alpine
- Runs bun run dev

Create a Makefile with commands:
- `make dev` - Start development environment
- `make down` - Stop all services
- `make logs` - Tail all logs
- `make db-shell` - Connect to PostgreSQL
- `make migrate` - Run database migrations
- `make test-api` - Run API tests
- `make test-frontend` - Run frontend tests

Create .env.example with all required variables for development.

Ensure the setup allows for:
- Hot reload on Rust code changes
- Hot module replacement for React
- Database persistence between restarts
```

---

## Prompt 1.5: Create Traefik Development Configuration

```text
Add Traefik reverse proxy to the Docker development environment.

Update docker-compose.dev.yml to add:

1. **traefik** service:
   - Image: traefik:v3.0
   - Expose ports 80 and 443
   - Dashboard on port 8081 (dev only)
   - Mount docker socket (read-only)
   - Enable Docker provider

2. Configure routing labels for:
   - `localhost` -> frontend
   - `api.localhost` -> api
   - `admin.localhost` -> frontend (admin routes)

3. Create traefik/traefik.dev.yml:
   - Enable Docker provider
   - Enable dashboard
   - HTTP entrypoint on 80
   - Log level: DEBUG

4. Update api and frontend services with Traefik labels:
   ```yaml
   labels:
     - "traefik.enable=true"
     - "traefik.http.routers.api.rule=Host(`api.localhost`)"
     - "traefik.http.services.api.loadbalancer.server.port=8080"
   ```

5. Add /etc/hosts entries to README:
   ```
   127.0.0.1 localhost api.localhost admin.localhost
   ```

Test that:
- http://localhost shows frontend
- http://api.localhost/health returns OK
- http://localhost:8081 shows Traefik dashboard
```

---

## Prompt 1.6: Create Project README and Documentation

```text
Create comprehensive project documentation.

Create README.md in project root with:

1. **Project Overview**
   - What example.com is
   - Tech stack summary
   - Project structure

2. **Quick Start**
   - Prerequisites
   - Clone and setup steps
   - How to run with Docker Compose
   - How to access the application

3. **Development**
   - Project structure explanation
   - How to add new API endpoints
   - How to add new frontend pages
   - Running tests
   - Code style/linting

4. **Environment Variables**
   - Table of all env vars
   - Which are required vs optional
   - Default values

5. **Available Commands**
   - All Makefile commands
   - Manual docker-compose commands
   - Cargo commands for API
   - bun commands for frontend

6. **Architecture Decisions**
   - Why Actix-Web
   - Why SQLx over other ORMs
   - JWT strategy explanation
   - Subdomain routing approach

Create CONTRIBUTING.md with:
- How to submit issues
- How to submit PRs
- Code review process
- Commit message format

Create docs/ARCHITECTURE.md with:
- System diagram (ASCII or link to diagram)
- Service responsibilities
- Data flow for key operations
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] `cargo build` succeeds in api/ directory
- [ ] `cargo test` passes
- [ ] `make dev` starts all services
- [ ] http://localhost:8080/health returns `{"status": "ok"}`
- [ ] Logs show structured JSON output
- [ ] Database connection pool works
- [ ] CORS headers present on API responses
- [ ] Traefik routes requests correctly

---

## Next Steps

Proceed to **[02-database-schema.md](./02-database-schema.md)** to implement the database schema.
