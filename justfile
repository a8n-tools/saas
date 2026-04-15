# SaaS Platform - Task Runner

# List available recipes
default:
    @just --list

export UID := `id -u`
export GID := `id -g`

# Use the per-user dev compose file
compose := "docker compose -f compose.dev.yml "
compose-local := "docker compose -f compose.dev.yml -f compose.dev.local.yml "

# Create .env files from dev/example defaults if they don't exist
[private]
ensure-env:
    @test -f .env || cp .env.example .env
    @test -f api/.env || cp api/.env.example api/.env
    @test -f frontend/.env || cp frontend/.env.example frontend/.env

# Development
# Start development environment on dev-01 via Traefik (foreground)
dev: ensure-env
    {{ compose }}up --build

# Start development environment on dev-01 via Traefik (detached)
dev-detach: ensure-env
    {{ compose }}up --build --detach
    @echo ""
    @echo "Services started!"

# Start development environment with localhost ports (foreground)
dev-local: ensure-env
    {{ compose-local }}up --build

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

# Release
# Create a release: bump major (vx.0.0) or minor version (v0.x.0), push branch, and print PR link
# After the PR is merged, create the release and tag in the Forgejo web UI
create-release bump:
    #!/usr/bin/env nu
    let bump = "{{ bump }}"
    if $bump not-in ["major" "minor"] {
        print $"(ansi red)Usage: just create-release <major|minor>(ansi reset)"
        exit 1
    }

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

    # Version agreement check
    let cargo_version = (open api/Cargo.toml | get package.version)
    let frontend_version = (open frontend/package.json | get version)
    let latest_tag = (git tag --list 'v*' | lines | sort --natural | last | str trim --left --char 'v')
    if $cargo_version != $frontend_version or $cargo_version != $latest_tag {
        print $"(ansi red)Error: version mismatch — all sources must agree before creating a release.(ansi reset)"
        print ""
        print $"  api/Cargo.toml:        v($cargo_version)"
        print $"  frontend/package.json: v($frontend_version)"
        print $"  latest git tag:        v($latest_tag)"
        print ""
        print "Fix the versions so all three match, then retry."
        exit 1
    }

    # Calculate next version
    let current = ($cargo_version | split row "." | each { into int })
    let next = match $bump {
        "major" => [$"($current.0 + 1)" "0" "0"],
        "minor" => [$"($current.0)" $"($current.1 + 1)" "0"],
    }
    let bare = ($next | str join ".")
    let tag = $"v($bare)"
    let release_branch = $"release/($tag)"

    # Create release branch, bump versions, and commit
    git checkout -b $release_branch
    open api/Cargo.toml | update package.version $bare | to toml | collect | save --force api/Cargo.toml
    open frontend/package.json | update version $bare | save --force frontend/package.json
    git add api/Cargo.toml frontend/package.json
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
    print $"Create PR:      ($base_url)/compare/main...($release_branch)"
    print $"After merging, create release ($tag) targeting main:"
    print $"  ($base_url)/releases/new"

# Cleanup
# Stop services and remove volumes
clean:
    {{ compose }}down --volumes --remove-orphans
    @echo "Volumes removed. Data has been cleared."
