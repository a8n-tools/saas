# SaaS Platform - Task Runner

# List available recipes
default:
    @just --list

export UID := `id -u`
export GID := `id -g`

# Use the per-user dev compose file
compose := "docker compose -f compose.dev.yml "

# Create .env files from dev/example defaults if they don't exist
[private]
ensure-env:
    @test -f .env || cp .env.example .env
    @test -f api/.env || cp api/.env.example api/.env
    @test -f frontend/.env || cp frontend/.env.example frontend/.env

# Development
# Start development environment (foreground)
dev: ensure-env
    {{ compose }}up --build

# Start development environment (detached)
dev-detach: ensure-env
    {{ compose }}up --build --detach
    @echo ""
    @echo "Services started!"
    @echo "  Frontend:  http://localhost:5173"
    @echo "  API:       http://localhost:18080"

# Generate the dev OIDC signing keypair if missing (kid=dev-2026, Ed25519).
# The repo intentionally does not ship these keys (commit d296c84); each
# instance must generate its own. Without them the API crash-loops with
# "Failed to read OIDC private key /run/secrets/oidc/dev-2026.pem".
[private]
ensure-oidc-keys:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f secrets/dev-2026.pem ] && [ -f secrets/dev-2026.pub.pem ]; then
        exit 0
    fi
    echo "Generating dev OIDC Ed25519 keypair (kid=dev-2026)..."
    if [ ! -d secrets ] || [ ! -w secrets ]; then
        docker run --rm -u 0 -v "$PWD:/work" alpine sh -c "mkdir -p /work/secrets && chown $(id -u):$(id -g) /work/secrets"
    fi
    docker run --rm -u "$(id -u):$(id -g)" -v "$PWD/secrets:/out" alpine/openssl genpkey -algorithm Ed25519 -out /out/dev-2026.pem
    docker run --rm -u "$(id -u):$(id -g)" -v "$PWD/secrets:/out" alpine/openssl pkey -in /out/dev-2026.pem -pubout -out /out/dev-2026.pub.pem
    chmod 600 secrets/dev-2026.pem
    chmod 644 secrets/dev-2026.pub.pem

# Start development environment on a8n.run (detached, Traefik-routed, for SSO testing)
dev-sso: ensure-env ensure-oidc-keys
    {{ compose }}up --build --detach
    @echo "  Frontend:  https://{{env('USER')}}-app.a8n.run"
    @echo "  API:       https://{{env('USER')}}-api.a8n.run"

# Stop all services
down:
    {{ compose }}down

# Tail all service logs
logs:
    {{ compose }}logs --follow

# Tail API logs only
logs-api:
    {{ compose }}logs --follow api

# Tail frontend logs only
logs-frontend:
    {{ compose }}logs --follow frontend

# Database
# Connect to PostgreSQL shell
db-shell:
    {{ compose }}exec postgres psql --username a8n --dbname a8n_platform

# Run database migrations
migrate:
    {{ compose }}exec api cargo sqlx migrate run

# Create a new migration
migrate-create name:
    {{ compose }}exec api cargo sqlx migrate add {{ name }}

# Testing
# Run API unit tests (no database required)
test-api:
    {{ compose }}exec -e GIT_COMMIT=dev api cargo test --lib

# Run frontend tests (single run, no watch)
test-frontend:
    {{ compose }}exec frontend bun run test:run

# Run API tests via docker exec (no compose dependency)
test-dev-api:
    docker exec saas-api-{{ `echo $USER` }} cargo test

# Run frontend tests via docker exec (no compose dependency)
test-dev-frontend:
    docker exec saas-frontend-{{ `echo $USER` }} bun run test:run

# Run all tests via docker exec (no compose dependency)
test-dev: test-dev-api test-dev-frontend

# Run all tests
test: test-api test-frontend

# Linting
# Run API clippy
lint-api:
    {{ compose }}exec api cargo clippy

# Run API formatter
fmt-api:
    {{ compose }}exec api cargo fmt

# Run frontend linter
lint-frontend:
    {{ compose }}exec frontend bun run lint

# Build
# Build all Docker images (dev)
build:
    {{ compose }}build

# Build API Docker image (dev)
build-api:
    {{ compose }}build api

# Build frontend Docker image (dev)
build-frontend:
    {{ compose }}build frontend

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

# ── Release ──────────────────────────────────────────────────────────────────

# Create a release: bump major (vx.0.0), minor (v0.x.0), or hotfix (v0.0.x), push branch, and print PR link
# After the PR is merged, the create-release workflow creates the tag and release automatically
create-release bump:
    #!/usr/bin/env nu
    let bump = "{{ bump }}"

    # Abort if there are uncommitted changes
    let status = git status --porcelain | str trim
    if ($status | is-not-empty) {
        print $"(ansi red)Working tree is dirty. Please stash or commit your changes first.(ansi reset)"
        exit 1
    }

    # Switch to main if not already there
    let branch = git branch --show-current | str trim
    if $branch != "main" {
        print $"Switching from ($branch) to main..."
        git checkout main
    }

    # Pull latest changes
    git pull --rebase origin main

    # Calculate next version
    let current = (open Cargo.toml | get package.version | split row "." | each { into int })
    let next = match $bump {
        "major" => [$"($current.0 + 1)" "0" "0"],
        "minor" => [$"($current.0)" $"($current.1 + 1)" "0"],
        "hotfix" => [$"($current.0)" $"($current.1)" $"($current.2 + 1)"],
        _ => { print $"(ansi red)Usage: just create-release <major|minor|hotfix>(ansi reset)"; exit 1 }
    }
    let bare = ($next | str join ".")
    let tag = $"v($bare)"
    let release_branch = $"release/($tag)"

    # Create release branch, bump version, and commit
    git checkout -b $release_branch
    open Cargo.toml | update package.version $bare | to toml | collect | save --force Cargo.toml
    git add Cargo.toml
    git commit --signoff --message $"Release ($tag)"

    # Push release branch
    git push --set-upstream origin $release_branch

    # Print PR and release links
    let remote = git remote get-url origin
    let base_url = if ($remote | str starts-with "ssh://") {
        $remote | str replace "ssh://git@" "https://" | str replace "git.a8n.run" "dev.a8n.run" | str replace ".git" ""
    } else {
        $remote | str replace --regex "git@([^:]+):" "https://$1/" | str replace "git.a8n.run" "dev.a8n.run" | str replace ".git" ""
    }
    print $"(ansi green)Pushed ($release_branch)(ansi reset)"
    print $"Create PR: ($base_url)/compare/main...($release_branch)"
    print $"After merging, the create-release workflow will tag and release ($tag) automatically."


# Cleanup
# Stop services and remove volumes (including oci + downloads caches)
clean:
    {{ compose }}down --volumes --remove-orphans
    -docker volume rm saas-oci-cache-$USER 2>/dev/null
    @echo "Volumes removed. Data has been cleared."
