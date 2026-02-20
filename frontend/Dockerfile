# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build arguments for environment variables
ARG VITE_API_URL=https://api.a8n.tools
ARG VITE_SHOW_BUSINESS_PRICING=false

# Set environment variables for build
ENV VITE_API_URL=$VITE_API_URL
ENV VITE_SHOW_BUSINESS_PRICING=$VITE_SHOW_BUSINESS_PRICING

# Build the application
RUN npm run build

# Production stage - Caddy to serve static files
FROM caddy:alpine

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
