# Build stage
FROM oven/bun:alpine AS builder

WORKDIR /app

# Build metadata (passed from CI; vite.config.ts reads these env vars)
ARG GIT_COMMIT=unknown
ARG GIT_TAG=unknown
ARG BUILD_DATE=unknown
ARG APP_VERSION=
ENV GIT_COMMIT=${GIT_COMMIT} \
    GIT_TAG=${GIT_TAG} \
    BUILD_DATE=${BUILD_DATE} \
    APP_VERSION=${APP_VERSION}

# Copy package files
COPY package.json bun.lock ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
RUN bun run build

# Production stage - Caddy to serve static files
FROM alpine:3.21

ARG GIT_COMMIT=unknown
ARG GIT_TAG=unknown
ARG BUILD_DATE=unknown

# Copy Caddy binary from official image
COPY --from=caddy:alpine /usr/bin/caddy /usr/bin/caddy

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Copy built assets from builder
COPY --from=builder /app/dist /usr/share/caddy

# Run as non-root user
RUN adduser -D -u 1001 appuser && \
    chown -R appuser:appuser /usr/share/caddy
USER appuser

LABEL org.opencontainers.image.source=https://dev.a8n.run/a8n-tools/saas \
      org.opencontainers.image.revision=${GIT_COMMIT} \
      org.opencontainers.image.version=${GIT_TAG} \
      org.opencontainers.image.created=${BUILD_DATE}

# Default port (configurable via PORT env var)
ENV PORT=8080
EXPOSE 8080

CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile"]
