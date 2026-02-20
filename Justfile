# Development
# Start development environment
dev:
    docker compose -f compose.dev.yml up -d --build
    @echo ""
    @echo "Services started!"
    @echo "  Frontend:  http://localhost:5173"
    @echo "  API:       http://localhost:18080"

# Stop all services
down:
    docker compose -f compose.dev.yml down

# Tail all service logs
logs:
    docker compose -f compose.dev.yml logs -f

# Tail API logs only
logs-api:
    docker compose -f compose.dev.yml logs -f api

# Tail frontend logs only
logs-frontend:
    docker compose -f compose.dev.yml logs -f frontend

# Database
# Connect to PostgreSQL shell
db-shell:
    docker compose -f compose.dev.yml exec postgres psql -U a8n -d a8n_platform

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
    cd frontend && npm test

# Run all tests
test: test-api test-frontend

# Build
# Build all Docker images
build:
    docker compose -f compose.dev.yml build

# Build API Docker image
build-api:
    docker compose -f compose.dev.yml build api

# Build frontend Docker image
build-frontend:
    docker compose -f compose.dev.yml build frontend

# Cleanup
# Stop services and remove volumes
clean:
    docker compose -f compose.dev.yml down -v
    @echo "Volumes removed. Data has been cleared."
