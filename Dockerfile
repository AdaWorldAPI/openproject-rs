# OpenProject RS - Multi-stage Dockerfile
# Optimized for small image size and fast builds

# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace configuration first (for better caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies first
RUN mkdir -p crates/op-server/src && \
    echo 'fn main() { println!("dummy"); }' > crates/op-server/src/main.rs

# Copy all crate Cargo.toml files for dependency resolution
COPY crates/op-core/Cargo.toml crates/op-core/
COPY crates/op-models/Cargo.toml crates/op-models/
COPY crates/op-contracts/Cargo.toml crates/op-contracts/
COPY crates/op-auth/Cargo.toml crates/op-auth/
COPY crates/op-services/Cargo.toml crates/op-services/
COPY crates/op-db/Cargo.toml crates/op-db/
COPY crates/op-queries/Cargo.toml crates/op-queries/
COPY crates/op-api/Cargo.toml crates/op-api/
COPY crates/op-notifications/Cargo.toml crates/op-notifications/
COPY crates/op-attachments/Cargo.toml crates/op-attachments/
COPY crates/op-journals/Cargo.toml crates/op-journals/
COPY crates/op-server/Cargo.toml crates/op-server/

# Create dummy lib.rs files for all crates
RUN for dir in op-core op-models op-contracts op-auth op-services op-db op-queries op-api op-notifications op-attachments op-journals; do \
    mkdir -p crates/$dir/src && \
    echo 'pub fn dummy() {}' > crates/$dir/src/lib.rs; \
    done

# Build dependencies (this layer will be cached)
RUN cargo build --release --package op-server 2>/dev/null || true

# Now copy actual source code
COPY crates/ crates/

# Touch all files to ensure they're rebuilt
RUN find crates -name "*.rs" -exec touch {} \;

# Build the actual application
RUN cargo build --release --package op-server

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash openproject

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/openproject-server /app/openproject-server

# Create directories for attachments and logs
RUN mkdir -p /var/openproject/assets /var/log/openproject && \
    chown -R openproject:openproject /var/openproject /var/log/openproject /app

# Switch to non-root user
USER openproject

# Environment defaults
ENV RUST_LOG=info,op_server=debug,op_api=debug \
    HOST=0.0.0.0 \
    PORT=8080 \
    OPENPROJECT_ATTACHMENTS_STORAGE_PATH=/var/openproject/assets

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Use tini as init process for proper signal handling
ENTRYPOINT ["/usr/bin/tini", "--"]

# Run the server
CMD ["/app/openproject-server"]
