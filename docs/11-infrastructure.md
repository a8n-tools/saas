# 11 - Infrastructure & Deployment

## Overview

This document contains prompts for setting up production infrastructure including Docker configuration, Traefik routing, and deployment procedures.

## Prerequisites
- All application code complete
- Domain configured
- Server provisioned

---

## Prompt 11.1: Production Docker Compose

```text
Create the production Docker Compose configuration.

Create docker-compose.yml:
```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v3.0
    container_name: a8n-traefik
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./traefik/traefik.yml:/etc/traefik/traefik.yml:ro
      - ./traefik/dynamic:/etc/traefik/dynamic:ro
      - traefik-certs:/letsencrypt
    networks:
      - a8n-network
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.dashboard.rule=Host(`traefik.a8n.tools`)"
      - "traefik.http.routers.dashboard.service=api@internal"
      - "traefik.http.routers.dashboard.middlewares=auth"
      - "traefik.http.middlewares.auth.basicauth.users=${TRAEFIK_DASHBOARD_AUTH}"

  api:
    build:
      context: ./api
      dockerfile: Dockerfile
    container_name: a8n-api
    restart: unless-stopped
    environment:
      - DATABASE_URL=postgres://a8n:${DB_PASSWORD}@postgres:5432/a8n_platform
      - JWT_PRIVATE_KEY_PATH=/app/keys/jwt_private.pem
      - JWT_PUBLIC_KEY_PATH=/app/keys/jwt_public.pem
      - STRIPE_SECRET_KEY=${STRIPE_SECRET_KEY}
      - STRIPE_WEBHOOK_SECRET=${STRIPE_WEBHOOK_SECRET}
      - STRIPE_PRICE_ID=${STRIPE_PRICE_ID}
      - SMTP_HOST=stalwart
      - SMTP_PORT=587
      - SMTP_USERNAME=${SMTP_USERNAME}
      - SMTP_PASSWORD=${SMTP_PASSWORD}
      - BASE_URL=https://app.a8n.tools
      - RUST_LOG=info
      - GLITCHTIP_DSN=${GLITCHTIP_DSN}
    volumes:
      - ./keys:/app/keys:ro
      - ./templates:/app/templates:ro
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - a8n-network
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.api.rule=Host(`api.a8n.tools`)"
      - "traefik.http.routers.api.tls=true"
      - "traefik.http.routers.api.tls.certresolver=letsencrypt"
      - "traefik.http.services.api.loadbalancer.server.port=8080"

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
      args:
        - VITE_API_URL=https://api.a8n.tools
    container_name: a8n-frontend
    restart: unless-stopped
    networks:
      - a8n-network
    labels:
      - "traefik.enable=true"
      # Landing page
      - "traefik.http.routers.landing.rule=Host(`a8n.tools`)"
      - "traefik.http.routers.landing.tls=true"
      - "traefik.http.routers.landing.tls.certresolver=letsencrypt"
      # App dashboard
      - "traefik.http.routers.app.rule=Host(`app.a8n.tools`)"
      - "traefik.http.routers.app.tls=true"
      - "traefik.http.routers.app.tls.certresolver=letsencrypt"
      # Admin panel
      - "traefik.http.routers.admin.rule=Host(`admin.a8n.tools`)"
      - "traefik.http.routers.admin.tls=true"
      - "traefik.http.routers.admin.tls.certresolver=letsencrypt"
      - "traefik.http.services.frontend.loadbalancer.server.port=80"

  postgres:
    image: postgres:16-alpine
    container_name: a8n-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_USER=a8n
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=a8n_platform
    volumes:
      - postgres-data:/var/lib/postgresql/data
    networks:
      - a8n-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U a8n -d a8n_platform"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: a8n-redis
    restart: unless-stopped
    volumes:
      - redis-data:/data
    networks:
      - a8n-network

  # Application containers
  rus:
    build: ./apps/rus
    container_name: a8n-rus
    restart: unless-stopped
    environment:
      - DATABASE_URL=postgres://rus:${RUS_DB_PASSWORD}@rus-db:5432/rus
      - JWT_PUBLIC_KEY_PATH=/app/keys/jwt_public.pem
    volumes:
      - ./keys/jwt_public.pem:/app/keys/jwt_public.pem:ro
    depends_on:
      - rus-db
    networks:
      - a8n-network
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.rus.rule=Host(`rus.a8n.tools`)"
      - "traefik.http.routers.rus.tls=true"
      - "traefik.http.routers.rus.tls.certresolver=letsencrypt"

  rus-db:
    image: postgres:16-alpine
    container_name: a8n-rus-db
    restart: unless-stopped
    environment:
      - POSTGRES_USER=rus
      - POSTGRES_PASSWORD=${RUS_DB_PASSWORD}
      - POSTGRES_DB=rus
    volumes:
      - rus-data:/var/lib/postgresql/data
    networks:
      - a8n-network

  rustylinks:
    build: ./apps/rustylinks
    container_name: a8n-rustylinks
    restart: unless-stopped
    environment:
      - DATABASE_URL=postgres://rustylinks:${RUSTYLINKS_DB_PASSWORD}@rustylinks-db:5432/rustylinks
      - JWT_PUBLIC_KEY_PATH=/app/keys/jwt_public.pem
    volumes:
      - ./keys/jwt_public.pem:/app/keys/jwt_public.pem:ro
    depends_on:
      - rustylinks-db
    networks:
      - a8n-network
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.rustylinks.rule=Host(`rustylinks.a8n.tools`)"
      - "traefik.http.routers.rustylinks.tls=true"
      - "traefik.http.routers.rustylinks.tls.certresolver=letsencrypt"

  rustylinks-db:
    image: postgres:16-alpine
    container_name: a8n-rustylinks-db
    restart: unless-stopped
    environment:
      - POSTGRES_USER=rustylinks
      - POSTGRES_PASSWORD=${RUSTYLINKS_DB_PASSWORD}
      - POSTGRES_DB=rustylinks
    volumes:
      - rustylinks-data:/var/lib/postgresql/data
    networks:
      - a8n-network

networks:
  a8n-network:
    driver: bridge

volumes:
  postgres-data:
  redis-data:
  rus-data:
  rustylinks-data:
  traefik-certs:
  stalwart-data:
```
```

---

## Prompt 11.2: Traefik Configuration

```text
Create Traefik configuration for production.

Create traefik/traefik.yml:
```yaml
api:
  dashboard: true

entryPoints:
  web:
    address: ":80"
    http:
      redirections:
        entryPoint:
          to: websecure
          scheme: https

  websecure:
    address: ":443"
    http:
      tls:
        certResolver: letsencrypt

providers:
  docker:
    endpoint: "unix:///var/run/docker.sock"
    exposedByDefault: false
    network: a8n-network
  file:
    directory: /etc/traefik/dynamic
    watch: true

certificatesResolvers:
  letsencrypt:
    acme:
      email: admin@a8n.tools
      storage: /letsencrypt/acme.json
      httpChallenge:
        entryPoint: web

log:
  level: INFO
  format: json

accessLog:
  format: json
  filters:
    statusCodes:
      - "400-599"
```

Create traefik/dynamic/security.yml:
```yaml
http:
  middlewares:
    security-headers:
      headers:
        browserXssFilter: true
        contentTypeNosniff: true
        frameDeny: true
        stsIncludeSubdomains: true
        stsPreload: true
        stsSeconds: 31536000
        customResponseHeaders:
          X-Robots-Tag: "noindex,nofollow,nosnippet,noarchive,notranslate,noimageindex"

    rate-limit:
      rateLimit:
        average: 100
        burst: 50
        period: 1m

    compress:
      compress: {}
```

Create traefik/dynamic/tls.yml:
```yaml
tls:
  options:
    default:
      minVersion: VersionTLS12
      cipherSuites:
        - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
        - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
        - TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
```
```

---

## Prompt 11.3: Production Dockerfiles

```text
Create optimized production Dockerfiles.

Create api/Dockerfile:
```dockerfile
# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Build application
COPY . .
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

COPY --from=builder /app/target/release/a8n-api /app/a8n-api
COPY --from=builder /app/migrations /app/migrations

RUN adduser -D -u 1000 appuser
USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

CMD ["/app/a8n-api"]
```

Create frontend/Dockerfile:
```dockerfile
# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

ARG VITE_API_URL
ENV VITE_API_URL=$VITE_API_URL

RUN npm run build

# Runtime stage
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:80/health || exit 1

CMD ["nginx", "-g", "daemon off;"]
```

Create frontend/nginx.conf:
```nginx
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }

    location / {
        try_files $uri $uri/ /index.html;
    }

    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;
}
```
```

---

## Prompt 11.4: Deployment Scripts

```text
Create deployment automation scripts.

Create scripts/deploy.sh:
```bash
#!/bin/bash
set -e

echo "🚀 Starting deployment..."

# Pull latest code
git pull origin main

# Build images
echo "📦 Building images..."
docker-compose build --parallel

# Run migrations
echo "🗄️ Running migrations..."
docker-compose run --rm api /app/a8n-api migrate

# Start services
echo "🔄 Starting services..."
docker-compose up -d

# Wait for health checks
echo "⏳ Waiting for services to be healthy..."
sleep 10

# Verify deployment
echo "✅ Verifying deployment..."
curl -sf https://api.a8n.tools/health > /dev/null && echo "API: OK" || echo "API: FAILED"
curl -sf https://app.a8n.tools > /dev/null && echo "Frontend: OK" || echo "Frontend: FAILED"

echo "🎉 Deployment complete!"
```

Create scripts/backup.sh:
```bash
#!/bin/bash
set -e

BACKUP_DIR="/backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo "📦 Backing up databases..."

# Platform database
docker exec a8n-postgres pg_dump -U a8n a8n_platform | gzip > "$BACKUP_DIR/platform.sql.gz"

# App databases
docker exec a8n-rus-db pg_dump -U rus rus | gzip > "$BACKUP_DIR/rus.sql.gz"
docker exec a8n-rustylinks-db pg_dump -U rustylinks rustylinks | gzip > "$BACKUP_DIR/rustylinks.sql.gz"

# Cleanup old backups (keep 30 days)
find /backups -type d -mtime +30 -exec rm -rf {} +

echo "✅ Backup complete: $BACKUP_DIR"
```

Create scripts/restore.sh:
```bash
#!/bin/bash
set -e

if [ -z "$1" ]; then
    echo "Usage: ./restore.sh <backup_dir>"
    exit 1
fi

BACKUP_DIR="$1"

echo "⚠️ This will overwrite current databases. Continue? (y/N)"
read confirm
if [ "$confirm" != "y" ]; then
    exit 1
fi

echo "📦 Restoring databases..."

gunzip -c "$BACKUP_DIR/platform.sql.gz" | docker exec -i a8n-postgres psql -U a8n a8n_platform
gunzip -c "$BACKUP_DIR/rus.sql.gz" | docker exec -i a8n-rus-db psql -U rus rus
gunzip -c "$BACKUP_DIR/rustylinks.sql.gz" | docker exec -i a8n-rustylinks-db psql -U rustylinks rustylinks

echo "✅ Restore complete!"
```

Make executable:
```bash
chmod +x scripts/*.sh
```
```

---

## Prompt 11.5: Environment Configuration

```text
Create environment configuration templates.

Create .env.example:
```bash
# Database
DB_PASSWORD=change_me_in_production

# Stripe
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRICE_ID=price_a8n_monthly_v1

# JWT Keys (generate with openssl)
# Run: openssl genpkey -algorithm Ed25519 -out jwt_private.pem
# Run: openssl pkey -in jwt_private.pem -pubout -out jwt_public.pem

# Email
SMTP_USERNAME=noreply@a8n.tools
SMTP_PASSWORD=change_me
STALWART_ADMIN_PASSWORD=change_me

# Traefik Dashboard (generate with: htpasswd -nb admin password)
TRAEFIK_DASHBOARD_AUTH=admin:$apr1$...

# Monitoring
GRAFANA_PASSWORD=change_me
GLITCHTIP_SECRET_KEY=change_me
GLITCHTIP_DSN=https://...@glitchtip.a8n.tools/1

# App Databases
RUS_DB_PASSWORD=change_me
RUSTYLINKS_DB_PASSWORD=change_me
```

Create scripts/generate-secrets.sh:
```bash
#!/bin/bash

echo "Generating secrets..."

# Generate random passwords
DB_PASSWORD=$(openssl rand -base64 32)
RUS_DB_PASSWORD=$(openssl rand -base64 32)
RUSTYLINKS_DB_PASSWORD=$(openssl rand -base64 32)
GRAFANA_PASSWORD=$(openssl rand -base64 32)
GLITCHTIP_SECRET_KEY=$(openssl rand -base64 64)
STALWART_ADMIN_PASSWORD=$(openssl rand -base64 32)

# Generate JWT keys
mkdir -p keys
openssl genpkey -algorithm Ed25519 -out keys/jwt_private.pem
openssl pkey -in keys/jwt_private.pem -pubout -out keys/jwt_public.pem
chmod 600 keys/jwt_private.pem
chmod 644 keys/jwt_public.pem

echo "Secrets generated. Add to .env file:"
echo ""
echo "DB_PASSWORD=$DB_PASSWORD"
echo "RUS_DB_PASSWORD=$RUS_DB_PASSWORD"
echo "RUSTYLINKS_DB_PASSWORD=$RUSTYLINKS_DB_PASSWORD"
echo "GRAFANA_PASSWORD=$GRAFANA_PASSWORD"
echo "GLITCHTIP_SECRET_KEY=$GLITCHTIP_SECRET_KEY"
echo "STALWART_ADMIN_PASSWORD=$STALWART_ADMIN_PASSWORD"
```
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] All containers start successfully
- [ ] Traefik routes to correct services
- [ ] HTTPS certificates obtained
- [ ] Health checks pass
- [ ] Database connections work
- [ ] Migrations run successfully
- [ ] Backup script works
- [ ] Restore script works
- [ ] All subdomains resolve correctly

---

## Next Steps

Proceed to **[12-monitoring.md](./12-monitoring.md)** to set up monitoring and observability.
