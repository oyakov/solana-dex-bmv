██████╗ ███╗   ███╗██╗   ██╗    ███████╗███████╗    ██████╗  ██████╗ ████████╗
██╔══██╗████╗ ████║██║   ██║    ██╔════╝██╔════╝    ██╔══██╗██╔═══██╗╚══██╔══╝
██████╔╝██╔████╔██║██║   ██║    █████╗  ███████╗    ██████╔╝██║   ██║   ██║   
██╔══██╗██║╚██╔╝██║╚██╗ ██╔╝    ██╔══╝  ╚════██║    ██╔══██╗██║   ██║   ██║   
██████╔╝██║ ╚═╝ ██║ ╚████╔╝     ███████╗███████║    ██████╔╝╚██████╔╝   ██║   
╚═════╝ ╚═╝     ╚═╝  ╚═══╝      ╚══════╝╚══════╝    ╚═════╝  ╚═════╝    ╚═╝
# Solana Dex (BMV)

## Architecture Overview

This project is an asyncio-driven Solana trading system that coordinates market data ingestion, decision logic, and transaction submission.

- **Event Loop Orchestrator**: Owns the asyncio loop, lifecycle hooks, and cancellation logic.
- **Market Data Pipeline**: Subscribes to Solana program accounts and external feeds, normalizes events, and pushes them onto internal queues.
- **Strategy Engine**: Consumes normalized events and produces trading intents (orders, cancels, rebalance signals).
- **Execution Layer**: Builds and signs transactions, optionally composes Jito bundles, and submits through RPC/MEV endpoints.
- **Safety & Risk Controls**: Enforces kill switch, circuit breaker, and sizing guards before any submission.
- **Observability**: Emits structured logs and optional metrics for runtime visibility.
- **TUI (Terminal UI)**: Real-time status, hotkeys, and safety toggles.

## Quick Start (Docker)

The fastest way to get started is using Docker Compose.

1. **Configure Environment**:
   Copy the example environment file and fill in your RPC URLs and keys.

   ```bash
   cp .env.example .env
   ```

2. **Launch with Docker Compose**:

   ```bash
   docker-compose up -d
   ```

3. **View Logs**:

   ```bash
   docker-compose logs -f
   ```

For more details, see [DOCKER.md](docs/DOCKER.md).

## Local Development Setup

### 1. Conda Environment (Recommended)

Follow the detailed instructions in [CONDA.md](docs/CONDA.md) to set up a Python 3.11 environment with all dependencies.

### 2. Manual Installation

```bash
pip install -r requirements.txt
```

## Running the Bot

To start the bot in dry-run mode (safest for testing):

```bash
python -m bot.main --dry-run
```

To start the production bot:

```bash
python -m bot.main
```

## Configuration

Define configuration via environment variables or a `.env` file. Recommended fields:

### Core

- `SOLANA_RPC_HTTP_URL`: Solana RPC HTTP endpoint.
- `SOLANA_RPC_WS_URL`: Solana RPC WebSocket endpoint.
- `WALLET_KEYPAIR_PATH`: Path to the keypair JSON file.
- `CLUSTER`: `mainnet-beta`, `devnet`, or `testnet`.
- `LOG_LEVEL`: `debug`, `info`, `warning`, `error`.

### Strategy & Risk

- `MAX_POSITION_SIZE`: Max allowed base size per market.
- `MAX_ORDER_NOTIONAL`: Max notional per order.
- `CIRCUIT_BREAKER_THRESHOLD`: Loss threshold before pausing trading.
- `CIRCUIT_BREAKER_WINDOW_SECS`: Time window for loss monitoring.
- `KILL_SWITCH_FILE`: Path to a sentinel file enabling immediate shutdown.

### Execution

- `TX_CONFIRM_TIMEOUT_SECS`: Timeout for confirmation await.
- `TX_MAX_RETRIES`: Retry count for failed submissions.
- `PRIORITY_FEE_MICROLAMPORTS`: Priority fee per CU.

### Jito Bundle

- `JITO_ENABLED`: `true` or `false`.
- `JITO_BLOCK_ENGINE_URL`: Block engine endpoint.
- `JITO_AUTH_KEYPAIR_PATH`: Auth keypair for block engine.
- `JITO_TIP_LAMPORTS`: Tip amount for bundle inclusion.
- `JITO_BUNDLE_TIMEOUT_SECS`: Bundle submission timeout.

### Observability

- `STRUCTLOG_JSON`: `true` to emit JSON events.
- `PROMETHEUS_ENABLED`: `true` to expose metrics.
- `PROMETHEUS_PORT`: Metrics bind port.

### TUI

- `TUI_ENABLED`: `true` to enable terminal UI.
- `TUI_REFRESH_MS`: Refresh cadence.

## Jito Bundle Prerequisites

- **Access**: Ensure you have access to a Jito block engine endpoint.
- **Auth Keypair**: Configure `JITO_AUTH_KEYPAIR_PATH` with the correct keypair.
- **Tip Strategy**: Set `JITO_TIP_LAMPORTS` based on current inclusion costs.
- **Bundle Limits**: Ensure bundle size and CU limits comply with Jito rules.

## Safety Controls

### Kill Switch

The kill switch halts all trading immediately. This is typically implemented by:

- Creating the file specified by `KILL_SWITCH_FILE`.
- The event loop checks for this sentinel each tick and cancels tasks when present.

### Circuit Breaker

The circuit breaker halts new order submissions after losses exceed a threshold.

- `CIRCUIT_BREAKER_THRESHOLD` triggers the breaker when losses exceed the limit.
- `CIRCUIT_BREAKER_WINDOW_SECS` defines the rolling window.
- When tripped, the system pauses and requires manual reset or cooldown.

## Observability & Monitoring

### Structlog JSON Events

When `STRUCTLOG_JSON=true`, logs are emitted as JSON objects suitable for ingestion into ELK, Loki, or Datadog.

- Use consistent event keys such as `event`, `market`, `position`, `latency_ms`, and `tx_sig`.

### Prometheus Metrics (Optional)

Enable the metrics endpoint with `PROMETHEUS_ENABLED=true`.

- Exposes counters for orders submitted, bundles accepted/rejected, and RPC latency.
- Bind port configured by `PROMETHEUS_PORT`.

## TUI Controls

When enabled, the terminal UI provides:

- **Pause/Resume** trading.
- **Toggle Kill Switch**.
- **View Circuit Breaker State** and reset it.
- **Inspect Positions** and active orders.
- **View RPC / Jito latency**.

Recommended hotkeys (customize in your TUI module):

- `p`: Pause/resume
- `k`: Toggle kill switch
- `c`: Reset circuit breaker
- `q`: Quit

## Testing Instructions

Run basic unit tests:

```bash
pytest
```

For advanced testing (Integration, Devnet, Jito), see [TESTING.md](docs/TESTING.md).

## Project Documentation

- [Trading Strategy Design](docs/Алгоритм динамического построения торговой сетки ордеров.md)
- [Requirements](docs/requirements/DEX Trading Bot — Требования.md)
- [Conda Setup](docs/CONDA.md)
- [Docker Guide](docs/DOCKER.md)
- [Testing Guide](docs/TESTING.md)
- [Agent Workflows](docs/WORKFLOWS.md)
