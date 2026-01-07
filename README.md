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

## Required Stack
- **Python** 3.10+ (asyncio features and type hints)
- **Solana RPC** endpoint (HTTP + WebSocket)
- **Jito** block engine access (optional, for bundles)
- **Redis or in-memory queues** (depending on your deployment choice)
- **Prometheus client** (optional, for metrics)

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

## Running the Asyncio Event Loop
This repository does not currently ship a runnable module entrypoint. If you need
to bootstrap the event loop, create a small wrapper in your own codebase or
scripts directory. A typical pattern looks like:

```python
import asyncio
from solana_dex_bmv.app import run

async def main():
    await run()

if __name__ == "__main__":
    asyncio.run(main())
```

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

## Observability
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
Run the full test suite:

```bash
pytest
```

Run specific modules:

```bash
pytest tests/test_strategy.py
```

Run lint checks (if configured):

```bash
ruff check .
```
