# Tech Context: BMV Eco System Market Making Bot

## Runtime Environment
- **Python**: 3.11+
- **Host**: Linux Server (e.g., Hetzner) or Docker.
- **Concurrency**: `asyncio` for non-blocking I/O.

## Core Libraries
- **Solana SDK**: `solana-py`, `anchorpy` for blockchain interaction.
- **Jito**: Jito Block Engine SDK for bundles.
- **Database**: `sqlite3` (via `aiosqlite`) for persisting bot state, trades, and PnL.
- **Configuration**: `PyYAML` and `python-dotenv`.
- **Logging**: `structlog` for structured JSON logging.
- **UI**: `Textual` or similar for the TUI (Terminal User Interface).

## Infrastructure & APIs
- **Solana RPC**: High-performance HTTP/WS endpoints (e.g., Helius, Triton, QuickNode).
- **Jito Block Engine**: Dedicated MEV-protected submission.
- **DEX Integrations**:
    - **OpenBook V2**: Maker orders and Order Book depth.
    - **Raydium V4**: AMM pool monitoring.
    - **Jupiter**: Taker orders and arbitrage.

## Development & Operations
- **Environment Management**: Conda (see `docs/CONDA.md`).
- **Containerization**: Docker & Docker Compose (see `docs/DOCKER.md`).
- **Testing**: `pytest` for unit and integration tests.
- **Metrics**: Prometheus & Grafana (optional).
