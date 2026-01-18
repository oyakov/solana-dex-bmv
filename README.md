██████╗ ███╗   ███╗██╗   ██╗    ███████╗███████╗    ██████╗  ██████╗ ████████╗
██╔══██╗████╗ ████║██║   ██║    ██╔════╝██╔════╝    ██╔══██╗██╔═══██╗╚══██╔══╝
██████╔╝██╔████╔██║██║   ██║    █████╗  ███████╗    ██████╔╝██║   ██║   ██║   
██╔══██╗██║╚██╔╝██║╚██╗ ██╔╝    ██╔══╝  ╚════██║    ██╔══██╗██║   ██║   ██║   
██████╔╝██║ ╚═╝ ██║ ╚████╔╝     ███████╗███████║    ██████╔╝╚██████╔╝   ██║   
╚═════╝ ╚═╝     ╚═╝  ╚═══╝      ╚══════╝╚══════╝    ╚═════╝  ╚═════╝    ╚═╝
# Solana Dex (BMV) - Rust Implementation

## Architecture Overview

This project is a high-performance Solana trading system implemented in Rust, coordinating market data ingestion, decision logic, and transaction submission.

- **Asynchronous Runtime**: Powered by `tokio` for efficient concurrent task management.
- **Market Data Pipeline**: Subscribes to Solana program accounts and external feeds, normalizes events.
- **Strategy Engine**: Processes normalized events and produces trading intents.
- **Execution Layer**: Builds, signs, and submits transactions, with Jito MEV protection support.
- **Safety & Risk Controls**: Integrated Circuit Breaker and Risk Manager to guard all actions.
- **Safety & Risk Controls**: Integrated Circuit Breaker and Risk Manager to guard all actions.
- **Observability**: Structured logging using `tracing` and real-time metrics via Prometheus, Grafana, and a modern Solana DEX Dashboard (v0.4.8).
- **Security Hardening**: Secure configuration via environment variables, masked sensitive data in logs, and non-root execution in Docker.

## Security & Secret Management

The system is designed for secure production deployment:

- **Environment Variables**: Sensitive data (keys, RPC URLs) should be provided via environment variables or a `.env` file. Environment variables ALWAYS override values in `config.yaml`.
- **Credential Masking**: Private keys and sensitive wallet addresses are automatically masked in `Debug` logs and output with `***MASKED***`.
- **Non-Root Execution**: In Docker, the application runs under a dedicated `botuser` to ensure minimal system privileges.
- **Network Isolation**: By default, observability ports (3000, 9090, 9000) are bound to `127.0.0.1`, requiring an SSH tunnel for remote access.

## Prerequisites

- Rust (Edition 2021)
- Docker & Docker Compose
- PostgreSQL 15+ (if running natively)
- Solana RPC Endpoint (Mainnet-beta)

## Getting Started

1. **Configure Environment**: Copy `.env.example` to `.env` and fill in your secrets.
   ```bash
   cp .env.example .env
   ```
   *Note: Ensure `DATABASE_URL` is set if you're not using the default Docker DB.*

2. **Run with Docker**:
   ```bash
   docker-compose up -d --build
   ```

3. **View Logs**:

   ```bash
   docker-compose logs -f
   ```

### Observability (Prometheus & Grafana)

The bot exposes Prometheus metrics on port `9000`. The Docker Compose setup includes pre-configured Prometheus and Grafana services.

1. **Access Grafana**: `http://localhost:3000` (Default login: `admin` / `admin`)
2. **Access Prometheus**: `http://localhost:9090`
3. **Metrics Endpoint**: `http://localhost:9000`

For more details, see [DOCKER.md](docs/DOCKER.md).

## Local Development Setup

### 1. Prerequisites

- Rust (latest stable)
- Cargo
- OpenSSL (on Linux)

### 2. Build

```powershell
cargo build --release
```

## Running the Bot

To start the bot:

```powershell
cargo run -- --config config.yaml
```

Define configuration via `config.yaml` or environment variables. Environment variables take precedence.

### Environment Variable Overrides

The following variables can be used to set sensitive fields:

- `RPC_PRIMARY_HTTP`: Solana RPC HTTP URL.
- `RPC_PRIMARY_WS`: Solana RPC WebSocket URL.
- `TOKEN_MINT`: Mint address to trade.
- `OPENBOOK_MARKET_ID`: OpenBook V2 Market ID.
- `WALLET_KEYPAIRS`: Comma-separated list of base58 keys or paths to keypair files.
- `USDC_WALLET_3`: Address of the fallback USDC wallet.
- `JITO_BUNDLER_URL`: Jito bundle submission URL.
- `DRY_RUN_ENABLED`: Set to `true` or `false`.
- `DATABASE_PATH`: Path to the SQLite database file.

### Yaml configuration (config.yaml)

- `token_mint`: Mint address of the token to trade.
- `openbook_market_id`: OpenBook V2 market ID.
- `rpc_endpoints.primary_http`: Solana RPC HTTP endpoint.
- `rpc_endpoints.primary_ws`: Solana RPC WebSocket endpoint.
- `wallets.multi_wallet.keypairs`: List of paths to keypair files (Recommended: use `WALLET_KEYPAIRS` env var).
- `dry_run.enabled`: Set to `true` to simulate trades without real execution.

## Testing Instructions

Run all tests:

```powershell
cargo test
```

For more details, see [TESTING.md](docs/TESTING.md).

## Project Documentation

- [**Memory Bank**](docs/MEMORY_BANK.md): Core references, architecture, and project status.
- [Requirements](docs/requirements/BMV%20Eco%20System%20Market%20Making%20Bot%20—%20Требования.md)
- [Docker Guide](docs/DOCKER.md)
- [Testing Guide](docs/TESTING.md)
- [Dashboard & Testing Guide (v0.4.8)](docs/v0.4.6_DASHBOARD_AND_TESTING.md)
- [Agent Workflows](docs/WORKFLOWS.md)
