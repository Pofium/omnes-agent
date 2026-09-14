# Multi-stage Dockerfile for Omnes Agent (Rust Core + KAG Engine + Flutter Web ADE)
# 
# Stage 1: Build Rust Backend
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install system build dependencies for Rust crates, SQLite, and C++ bindings
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    git \
    g++ \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy backend workspace manifests and sources
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
COPY backend/crates ./backend/crates
COPY backend/apps ./backend/apps
COPY backend/src ./backend/src
COPY backend/build.rs ./backend/
COPY backend/locales.toml ./backend/
COPY backend/wit ./backend/wit

# Copy static web assets for embedding and serving
COPY web/dist /app/web/dist
COPY backend/web/dist /app/backend/web/dist

WORKDIR /app/backend

# Compile release binary for omnesagent
RUN cargo build --release --bin omnesagent

# -------------------------------------------------------------
# Stage 2: Minimal Runtime Container
# -------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime utilities needed by agent tools (git, curl, sqlite3, certificates)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    sqlite3 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Setup non-root system user and persistent volume mount point
RUN useradd -r -s /bin/bash -m -d /home/omnesagent omnesagent \
    && mkdir -p /app /app/web/dist /data \
    && chown -R omnesagent:omnesagent /app /data /home/omnesagent

# Copy built binary from builder
COPY --from=builder --chown=omnesagent:omnesagent /app/backend/target/release/omnesagent /usr/local/bin/omnesagent

# Copy pre-built Flutter Web assets for gateway dashboard
COPY --from=builder --chown=omnesagent:omnesagent /app/web/dist /app/web/dist

USER omnesagent
WORKDIR /app

# Runtime configuration
ENV OMNESAGENT_CONFIG_DIR=/data \
    OMNESAGENT_DATA_DIR=/data \
    OMNESAGENT_GATEWAY_HOST=0.0.0.0 \
    OMNESAGENT_GATEWAY_PORT=42617 \
    OMNESAGENT_GATEWAY_ALLOW_PUBLIC_BIND=true \
    RUST_LOG=info

# Gateway REST & WebSocket Port
EXPOSE 42617

# Volume for long-term memory (SQLite brain.db), KAG indices, and configs
VOLUME ["/data"]

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:42617/health || exit 1

# Launch Gateway daemon by default
ENTRYPOINT ["omnesagent"]
CMD ["gateway", "start"]
