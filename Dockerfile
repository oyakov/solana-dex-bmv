# Builder stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy project files
COPY . .

# Build the application
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/solana-dex-bmv /app/solana-dex-bmv
COPY --from=builder /app/config.yaml /app/config.yaml

# Run the bot
ENTRYPOINT ["/app/solana-dex-bmv"]
CMD ["--config", "/app/config.yaml"]
