.PHONY: help dev down logs db-shell migrate test-api test-frontend clean build

# Default target
help:
	@echo "a8n.tools Development Commands"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Development:"
	@echo "  dev            Start development environment"
	@echo "  down           Stop all services"
	@echo "  logs           Tail all service logs"
	@echo "  logs-api       Tail API logs only"
	@echo "  logs-frontend  Tail frontend logs only"
	@echo ""
	@echo "Database:"
	@echo "  db-shell       Connect to PostgreSQL shell"
	@echo "  migrate        Run database migrations"
	@echo "  migrate-create Create a new migration (NAME=migration_name)"
	@echo ""
	@echo "Testing:"
	@echo "  test-api       Run API tests"
	@echo "  test-frontend  Run frontend tests"
	@echo "  test           Run all tests"
	@echo ""
	@echo "Build:"
	@echo "  build          Build all Docker images"
	@echo "  build-api      Build API Docker image"
	@echo "  build-frontend Build frontend Docker image"
	@echo ""
	@echo "Cleanup:"
	@echo "  clean          Stop services and remove volumes"

# Development
dev:
	docker compose -f docker-compose.dev.yml up -d
	@echo ""
	@echo "Services started!"
	@echo "  Frontend:  http://localhost:5173"
	@echo "  API:       http://localhost:8080"
	@echo "  Traefik:   http://localhost:8081"
	@echo ""
	@echo "With Traefik:"
	@echo "  Frontend:  http://localhost"
	@echo "  API:       http://api.localhost"

down:
	docker compose -f docker-compose.dev.yml down

logs:
	docker compose -f docker-compose.dev.yml logs -f

logs-api:
	docker compose -f docker-compose.dev.yml logs -f api

logs-frontend:
	docker compose -f docker-compose.dev.yml logs -f frontend

# Database
db-shell:
	docker compose -f docker-compose.dev.yml exec postgres psql -U a8n -d a8n_platform

migrate:
	cd api && cargo sqlx migrate run

migrate-create:
	@if [ -z "$(NAME)" ]; then \
		echo "Error: NAME is required. Usage: make migrate-create NAME=migration_name"; \
		exit 1; \
	fi
	cd api && cargo sqlx migrate add $(NAME)

# Testing
test-api:
	cd api && cargo test

test-frontend:
	cd frontend && npm test

test: test-api test-frontend

# Build
build:
	docker compose -f docker-compose.dev.yml build

build-api:
	docker compose -f docker-compose.dev.yml build api

build-frontend:
	docker compose -f docker-compose.dev.yml build frontend

# Cleanup
clean:
	docker compose -f docker-compose.dev.yml down -v
	@echo "Volumes removed. Data has been cleared."
