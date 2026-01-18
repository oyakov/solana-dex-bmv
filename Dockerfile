# Builder stage with cargo-chef for dependency caching
FROM rustlang/rust:nightly-bookworm-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Planner stage - extract dependency recipe
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Cacher stage - build dependencies only (cached unless Cargo.toml changes)
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN cargo chef cook --release --recipe-path recipe.json

# Builder stage - build actual application
FROM chef AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Final stage - minimal production image
FROM debian:bookworm-slim

# Create a non-root user
RUN groupadd -r botgroup && useradd -r -g botgroup -m -s /sbin/nologin botuser

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/solana-dex-bmv /app/solana-dex-bmv
COPY Cargo.toml /app/Cargo.toml

# Set ownership to botuser
RUN chown -R botuser:botgroup /app

# Switch to non-root user
USER botuser

# Run the bot
CMD ["/app/solana-dex-bmv"]
