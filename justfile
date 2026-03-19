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
# Run API unit tests (no database required)
test-api:
    docker compose exec -e GIT_COMMIT=dev api cargo test --lib

# Run frontend tests (single run, no watch)
test-frontend:
    docker compose exec frontend bun run test:run

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

# Release
# Create a release: bump major (vx.0.0) or minor version (v0.x.0), commit, tag, and push
create-release bump:
    #!/usr/bin/env nu
    let bump = "{{ bump }}"
    if $bump not-in ["major" "minor"] {
        print $"(ansi red)Usage: just create-release <major|minor>(ansi reset)"
        exit 1
    }
    let cargo_version = (open api/Cargo.toml | get package.version)
    let frontend_version = (open frontend/package.json | get version)
    let latest_tag = (git tag --list 'v*' | lines | sort --natural | last | str trim --left --char 'v')
    # All three sources must agree before we proceed
    if $cargo_version != $frontend_version or $cargo_version != $latest_tag {
        print $"(ansi red)Error: version mismatch — all sources must agree before creating a release.(ansi reset)"
        print ""
        print $"  api/Cargo.toml:      v($cargo_version)"
        print $"  frontend/package.json: v($frontend_version)"
        print $"  latest git tag:        v($latest_tag)"
        print ""
        print "Fix the versions so all three match, then retry."
        exit 1
    }
    let current = ($cargo_version | split row "." | each { into int })
    let next = match $bump {
        "major" => [$"($current.0 + 1)" "0" "0"],
        "minor" => [$"($current.0)" $"($current.1 + 1)" "0"],
    }
    let bare = ($next | str join ".")
    let tag = $"v($bare)"
    open api/Cargo.toml | update package.version $bare | to toml | collect | save --force api/Cargo.toml
    open frontend/package.json | update version $bare | save --force frontend/package.json
    git add api/Cargo.toml frontend/package.json
    git commit --signoff --message $"Release ($tag)"
    git tag --annotate $tag --message $"Release ($tag)"
    git push --follow-tags
    print $"Released ($tag)"

# Test the release flow: create major release, cancel CI, delete tag, and revert commit (requires FORGEJO_TOKEN)
test-release:
    #!/usr/bin/env nu
    let token = ($env | get --ignore-errors FORGEJO_TOKEN | default "")
    if ($token | is-empty) { print $"(ansi red)FORGEJO_TOKEN env var required(ansi reset)"; exit 1 }
    let current = (open api/Cargo.toml | get package.version | split row "." | each { into int })
    let bare = $"($current.0 + 1).0.0"
    let tag = $"v($bare)"
    just create-release major
    print "Waiting for CI to pick up the tag..."
    sleep 5sec
    let headers = {Authorization: $"token ($token)"}
    let runs = (http get --headers $headers "https://dev.a8n.run/api/v1/repos/a8n-tools/saas/actions/runs")
    let matched = ($runs.workflow_runs | where prettyref == $tag)
    if ($matched | is-empty) {
        print $"(ansi yellow)No workflow run found for ($tag) — skipping cancel(ansi reset)"
    } else {
        let run_id = ($matched | first | get id)
        try {
            http post --headers $headers --content-type "application/json" $"https://dev.a8n.run/api/v1/repos/a8n-tools/saas/actions/runs/($run_id)/cancel" {}
            print $"Cancelled workflow run ($run_id)"
        } catch {
            print $"(ansi yellow)Could not cancel run ($run_id) — may have already completed(ansi reset)"
        }
    }
    ^git tag --delete $tag
    ^git push origin --delete $tag
    ^git revert --no-edit HEAD
    ^git push
    print $"Done — ($tag) cleaned up"

# Cleanup
# Stop services and remove volumes
clean:
    docker compose down --volumes --remove-orphans
    @echo "Volumes removed. Data has been cleared."
