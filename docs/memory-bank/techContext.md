# Tech Context: BMV Eco System Market Making Bot

## Runtime Environment
- **Rust**: Latest stable (1.75+)
- **Host**: Linux Server (Regxa-2core2gigs) or Docker.
- **Concurrency**: `tokio` for non-blocking I/O and async runtime.

## Core Libraries
- **Solana SDK**: `solana-sdk`, `solana-client` for blockchain interaction.
- **Jito**: Custom implementation for Jito Block Engine bundle submission.
- **Database**: `sqlx` with **PostgreSQL** for persisting bot state, trades, and price history.
- **Configuration**: `serde_yaml` and `dotenvy`.
- **Logging**: `tracing` and `tracing-subscriber` for structured logging.
- **Error Handling**: `anyhow`.

## Infrastructure & APIs
- **Solana RPC**: High-performance HTTP/WS endpoints.
- **Jito Block Engine**: Dedicated MEV-protected bundle submission.
- **External APIs**:
    - **Binance API**: Used for SOL/USDC historical price backfill.
- **DEX Integrations**:
    - **OpenBook V2**: Maker orders and Order Book depth.
    - **Raydium V4**: AMM pool monitoring (planned).

## Development & Operations
- **Build System**: Cargo.
- **Containerization**: Docker & Docker Compose (Non-root user).
- **Testing**: Native Rust test runner (`cargo test`).
- **Metrics**: Prometheus and Grafana (Live Dashboards for BMV and SOL/USDC).
