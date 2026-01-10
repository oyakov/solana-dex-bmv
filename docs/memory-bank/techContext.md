# Tech Context: BMV Eco System Market Making Bot

## Runtime Environment
- **Rust**: Latest stable (1.75+)
- **Host**: Linux Server (e.g., Hetzner) or Docker.
- **Concurrency**: `tokio` for non-blocking I/O and async runtime.

## Core Libraries
- **Solana SDK**: `solana-sdk`, `solana-client` for blockchain interaction.
- **Jito**: Jito Block Engine support (custom implementation or crate).
- **Database**: `sqlx` with SQLite for persisting bot state, trades, and PnL.
- **Configuration**: `serde_yaml` and `dotenvy`.
- **Logging**: `tracing` and `tracing-subscriber` for structured logging.
- **Error Handling**: `anyhow` and `thiserror`.

## Infrastructure & APIs
- **Solana RPC**: High-performance HTTP/WS endpoints (e.g., Helius, Triton, QuickNode).
- **Jito Block Engine**: Dedicated MEV-protected submission.
- **DEX Integrations**:
    - **OpenBook V2**: Maker orders and Order Book depth.
    - **Raydium V4**: AMM pool monitoring.
    - **Jupiter**: Taker orders and arbitrage.

## Development & Operations
- **Build System**: Cargo.
- **Containerization**: Docker & Docker Compose (Non-root user, local-only port binding).
- **Testing**: Native Rust test runner (`cargo test`).
- **Metrics**: Prometheus (port 9000, 127.0.0.1 binding) and masked secret logging.
