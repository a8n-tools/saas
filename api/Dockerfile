# Build stage
FROM rust:1.92-bookworm AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

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
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r a8n && useradd -r -g a8n a8n

# Copy the binary from builder
COPY --from=builder /app/target/release/a8n-api /app/a8n-api

# Copy migrations and templates if they exist
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/templates /app/templates

# Set ownership
RUN chown -R a8n:a8n /app

USER a8n

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
CMD ["/app/a8n-api"]
