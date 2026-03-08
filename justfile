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
    docker compose --file compose.yml up -d --build
    @echo ""
    @echo "Services started!"
    @echo "  Frontend:  http://localhost:5173"
    @echo "  API:       http://localhost:18080"

# Stop all services
down:
    docker compose --file compose.yml down

# Tail all service logs
logs:
    docker compose --file compose.yml logs --follow

# Tail API logs only
logs-api:
    docker compose --file compose.yml logs --follow api

# Tail frontend logs only
logs-frontend:
    docker compose --file compose.yml logs --follow frontend

# Database
# Connect to PostgreSQL shell
db-shell:
    docker compose --file compose.yml exec postgres psql --username a8n --dbname a8n_platform

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
# Build all Docker images (dev)
build:
    docker compose --file compose.yml build

# Build API Docker image (dev)
build-api:
    docker compose --file compose.yml build api

# Build frontend Docker image (dev)
build-frontend:
    docker compose --file compose.yml build frontend

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
    docker compose --file compose.yml down -v
    @echo "Volumes removed. Data has been cleared."
