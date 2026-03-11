# SaaS Platform - Task Runner

# List available recipes
default:
    @just --list

export UID := `id -u`
export GID := `id -g`

# Create .env files from dev/example defaults if they don't exist
[private]
ensure-env:
    @test -f .env || cp .env.example .env
    @test -f api/.env || cp api/.env.example api/.env
    @test -f frontend/.env || cp frontend/.env.example frontend/.env

# Development
# Start development environment (foreground)
dev: ensure-env
    docker compose up --build

# Start development environment (detached)
dev-detach: ensure-env
    docker compose up --build --detach
    @echo ""
    @echo "Services started!"
    @echo "  Frontend:  http://localhost:5173"
    @echo "  API:       http://localhost:18080"

# Stop all services
down:
    docker compose down

# Tail all service logs
logs:
    docker compose logs --follow

# Tail API logs only
logs-api:
    docker compose logs --follow api

# Tail frontend logs only
logs-frontend:
    docker compose logs --follow frontend

# Database
# Connect to PostgreSQL shell
db-shell:
    docker compose exec postgres psql --username a8n --dbname a8n_platform

# Run database migrations
migrate:
    docker compose exec api cargo sqlx migrate run

# Create a new migration
migrate-create name:
    docker compose exec api cargo sqlx migrate add {{ name }}

# Testing
# Run API tests
test-api:
    docker compose exec api cargo test

# Run frontend tests
test-frontend:
    docker compose exec frontend bun test

# Run all tests
test: test-api test-frontend

# Linting
# Run API clippy
lint-api:
    docker compose exec api cargo clippy

# Run API formatter
fmt-api:
    docker compose exec api cargo fmt

# Run frontend linter
lint-frontend:
    docker compose exec frontend bun run lint

# Build
# Build all Docker images (dev)
build:
    docker compose build

# Build API Docker image (dev)
build-api:
    docker compose build api

# Build frontend Docker image (dev)
build-frontend:
    docker compose build frontend

# Build API Docker image for validation (builder stage only)
check-docker-api:
    docker buildx build --target builder -t saas-api:check -f oci-build/api/Dockerfile api/

# Build frontend Docker image for validation
check-docker-frontend:
    docker buildx build -t saas-frontend:check -f oci-build/frontend/Dockerfile frontend/

# Build API Docker image
build-docker-api:
    docker buildx build -t saas-api:local -f oci-build/api/Dockerfile api/

# Build frontend Docker image
build-docker-frontend:
    docker buildx build -t saas-frontend:local -f oci-build/frontend/Dockerfile frontend/

# Cleanup
# Stop services and remove volumes
clean:
    docker compose down --volumes --remove-orphans
    @echo "Volumes removed. Data has been cleared."
