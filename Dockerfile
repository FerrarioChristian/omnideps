# ==========================================
# STAGE 1: Build the Visualizer Frontend
# ==========================================
FROM node:22-slim AS frontend-builder
WORKDIR /app/visualizer-svelte

COPY visualizer-svelte/package*.json ./
RUN npm ci

COPY visualizer-svelte ./
RUN npm run build

# ==========================================
# STAGE 2: Build the Omnideps Rust Binary
# ==========================================
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Install build dependencies for Tree-sitter C parsers
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY primitives.json ./
COPY docs/ ./docs/
COPY tests/ ./tests/
COPY src/ ./src/
COPY --from=frontend-builder /app/visualizer-svelte/build ./visualizer-svelte/build

RUN cargo build --release

# ==========================================
# STAGE 3: Minimal Runtime Container
# ==========================================
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Run as non-root user for security
RUN useradd -m -u 1001 omniuser
USER omniuser

COPY --from=builder /app/target/release/omnideps /usr/local/bin/omnideps

EXPOSE 3000

ENTRYPOINT ["omnideps"]
CMD ["serve", "--port", "3000"]
