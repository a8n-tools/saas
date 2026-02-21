# Build stage
FROM oven/bun:alpine AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lockb ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY . .

# Build arguments for environment variables
ARG VITE_API_URL=https://api.a8n.tools
ARG VITE_SHOW_BUSINESS_PRICING=false

# Set environment variables for build
ENV VITE_API_URL=$VITE_API_URL
ENV VITE_SHOW_BUSINESS_PRICING=$VITE_SHOW_BUSINESS_PRICING

# Build the application
RUN bun run build

# Production stage - Caddy to serve static files
FROM alpine:3.21

# Copy Caddy binary from official image
COPY --from=caddy:alpine /usr/bin/caddy /usr/bin/caddy

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Copy built assets from builder
COPY --from=builder /app/dist /usr/share/caddy

# Default port (configurable via PORT env var)
ENV PORT=8080
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -q --spider http://localhost:${PORT}/health || exit 1

CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile"]
