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
- **Security**: `argon2` for password hashing and `jsonwebtoken` for API authentication.
- **Error Handling**: `anyhow`.

## Infrastructure & APIs
- **Solana RPC**: High-performance HTTP/WS endpoints.
- **Jito Block Engine**: Dedicated MEV-protected bundle submission.
- **External APIs**:
    - **RugCheck API**: Used for automated security scanning of token mints.
    - **Binance API**: Used for SOL/USDC historical price backfill.
- **DEX Integrations**:
    - **OpenBook V1/V2**: Full support for Serum V3 and OpenBook V2 protocols.
    - **Raydium V4**: AMM pool monitoring for liquidity analysis.

## Development & Operations
- **Build System**: Cargo.
- **Containerization**: Docker & Docker Compose (Non-root user).
- **Frontend**: React-based dashboard with **D3.js** for high-performance market visualizations (Orderbook, Imbalance, Liquidity).
- **Testing**: Native Rust test runner and browser automation for dashboard verification.
- **Metrics**: Prometheus and Grafana (Enterprise-grade health monitoring).

