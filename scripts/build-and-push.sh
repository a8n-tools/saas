#!/bin/bash
# Build and push Docker images to Codeberg Container Registry
# Usage: ./scripts/build-and-push.sh [api|frontend|all] [version]

set -e

REGISTRY="codeberg.org/a8n-tools"
VERSION="${2:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if logged in to Codeberg registry
check_login() {
    if ! docker info 2>/dev/null | grep -q "codeberg.org"; then
        log_warn "You may need to login to Codeberg registry first:"
        echo "  docker login codeberg.org"
        echo ""
        read -p "Continue anyway? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

build_api() {
    log_info "Building API image..."
    docker build \
        -t "${REGISTRY}/saas-api:${VERSION}" \
        -t "${REGISTRY}/saas-api:latest" \
        -f api/Dockerfile \
        api/
    log_info "API image built: ${REGISTRY}/saas-api:${VERSION}"
}

build_frontend() {
    log_info "Building Frontend image..."
    docker build \
        --build-arg VITE_API_URL=https://api.a8n.tools \
        --build-arg VITE_SHOW_BUSINESS_PRICING=false \
        -t "${REGISTRY}/saas-frontend:${VERSION}" \
        -t "${REGISTRY}/saas-frontend:latest" \
        -f frontend/Dockerfile \
        frontend/
    log_info "Frontend image built: ${REGISTRY}/saas-frontend:${VERSION}"
}

push_api() {
    log_info "Pushing API image to Codeberg..."
    docker push "${REGISTRY}/saas-api:${VERSION}"
    if [ "$VERSION" != "latest" ]; then
        docker push "${REGISTRY}/saas-api:latest"
    fi
    log_info "API image pushed!"
}

push_frontend() {
    log_info "Pushing Frontend image to Codeberg..."
    docker push "${REGISTRY}/saas-frontend:${VERSION}"
    if [ "$VERSION" != "latest" ]; then
        docker push "${REGISTRY}/saas-frontend:latest"
    fi
    log_info "Frontend image pushed!"
}

case "${1:-all}" in
    api)
        check_login
        build_api
        push_api
        ;;
    frontend)
        check_login
        build_frontend
        push_frontend
        ;;
    all)
        check_login
        build_api
        build_frontend
        push_api
        push_frontend
        ;;
    build-only)
        build_api
        build_frontend
        log_info "Images built locally (not pushed)"
        ;;
    *)
        echo "Usage: $0 [api|frontend|all|build-only] [version]"
        echo ""
        echo "Commands:"
        echo "  api         Build and push API image only"
        echo "  frontend    Build and push Frontend image only"
        echo "  all         Build and push all images (default)"
        echo "  build-only  Build images locally without pushing"
        echo ""
        echo "Options:"
        echo "  version     Tag version (default: latest)"
        echo ""
        echo "Examples:"
        echo "  $0                    # Build and push all with 'latest' tag"
        echo "  $0 api v1.0.0         # Build and push API with 'v1.0.0' tag"
        echo "  $0 build-only         # Build all images locally"
        exit 1
        ;;
esac

log_info "Done!"
