# Build stage
FROM rust:1.93-alpine AS builder

WORKDIR /build

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

# Copy cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src for dependency compilation
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release && rm -rf src target/release/deps/a8n_api*

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:3.21

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Create non-root user
RUN adduser -D -u 1001 appuser

# Copy the binary from builder
COPY --from=builder /build/target/release/a8n-api /app/a8n-api

# Copy migrations and templates
COPY --from=builder /build/migrations /app/migrations
COPY --from=builder /build/templates /app/templates

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

LABEL org.opencontainers.image.source=https://dev.a8n.run/a8n-tools/saas

EXPOSE 8080

ENTRYPOINT ["/app/a8n-api"]
