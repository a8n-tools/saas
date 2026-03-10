# syntax=docker/dockerfile:1

FROM oven/bun:alpine
WORKDIR /app

# Layer 1: Dependencies (cached unless package.json changes)
COPY package.json bun.lock ./
RUN --mount=type=cache,target=/root/.bun/install/cache \
    bun install

# Make /app and node_modules writable so the non-root user can create .vite/ at runtime
RUN chmod 777 /app /app/node_modules

# Source code is bind-mounted at runtime, not copied
EXPOSE 5173
CMD ["bun", "run", "dev", "--", "--host", "0.0.0.0"]
