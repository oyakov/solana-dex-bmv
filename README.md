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
- **Observability**: Structured logging using `tracing` and real-time metrics via Prometheus & Grafana.

## Quick Start (Docker)

The fastest way to get started is using Docker Compose.

1. **Configure Environment**:
   Copy the example environment file and fill in your RPC URLs and keys.

   ```powershell
   copy .env.example .env
   ```

2. **Launch with Docker Compose**:

   ```powershell
   docker-compose up -d --build
   ```

3. **View Logs**:

   ```powershell
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

## Configuration

Define configuration via `config.yaml` or environment variables.

### Core Fields

- `token_mint`: Mint address of the token to trade.
- `openbook_market_id`: OpenBook V2 market ID.
- `rpc_endpoints.primary_http`: Solana RPC HTTP endpoint.
- `rpc_endpoints.primary_ws`: Solana RPC WebSocket endpoint.
- `wallets.multi_wallet.keypairs`: List of paths to keypair files for rotation.
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
- [Agent Workflows](docs/WORKFLOWS.md)
