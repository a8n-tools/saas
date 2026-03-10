# syntax=docker/dockerfile:1

FROM rust:1.88
WORKDIR /app

# Install dependencies needed for cargo-watch and sqlx
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for hot reloading
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-watch

# Copy cargo files first for better caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy src to compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached unless Cargo.toml/lock changes)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build 2>/dev/null || true

# Remove dummy src
RUN rm -rf src

# Pre-create target dir so Docker initializes the named volume with 777 permissions
RUN mkdir -p /app/target && chmod -R 777 /app

# Source code is bind-mounted at runtime, not copied
CMD ["cargo", "watch", "-x", "run"]
