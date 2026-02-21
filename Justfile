# JUST_DIR is the directory of the current Justfile
export JUST_DIR := justfile_directory()
# JUST_CURRENT_DIR it the invocation directory where the just command was run.
export JUST_INVOCATION_DIR := invocation_directory_native()


# default recipe to display help information
# just list will fallback to the parent justfile and list recipes only in the parent justfile.
# just --list returns only recipes in this justfile.
default:
	@just --list


# list the just recipes
list:
	@just --list

# Development
# Start development environment
dev:
    docker compose --file compose.dev.yml up -d --build
    @echo ""
    @echo "Services started!"
    @echo "  Frontend:  http://localhost:5173"
    @echo "  API:       http://localhost:18080"

# Stop all services
down:
    docker compose --file compose.dev.yml down

# Tail all service logs
logs:
    docker compose --file compose.dev.yml logs --follow

# Tail API logs only
logs-api:
    docker compose --file compose.dev.yml logs --follow api

# Tail frontend logs only
logs-frontend:
    docker compose --file compose.dev.yml logs --follow frontend

# Database
# Connect to PostgreSQL shell
db-shell:
    docker compose --file compose.dev.yml exec postgres psql -U a8n -d a8n_platform

# Run database migrations
migrate:
    cd api && cargo sqlx migrate run

# Create a new migration
migrate-create name:
    cd api && cargo sqlx migrate add {{ name }}

# Testing
# Run API tests
test-api:
    cd api && cargo test

# Run frontend tests
test-frontend:
    cd frontend && bun test

# Run all tests
test: test-api test-frontend

# Build
# Build all Docker images
build:
    docker compose --file compose.dev.yml build

# Build API Docker image
build-api:
    docker compose --file compose.dev.yml build api

# Build frontend Docker image
build-frontend:
    docker compose --file compose.dev.yml build frontend

# Cleanup
# Stop services and remove volumes
clean:
    docker compose --file compose.dev.yml down -v
    @echo "Volumes removed. Data has been cleared."
