# System Patterns: BMV Eco System Market Making Bot

## Architecture Overview
The system is built as a high-performance, asynchronous, event-driven trading engine using Rust and the `tokio` runtime. It follows a modular, service-oriented structure.

### 1. Core Lifecycle & Infrastructure
- **Central Core**: Powered by `tokio` (v1.x).
- **Infrastructure Layer**: Located in `src/infra/`, handles direct external integrations:
    - `SolanaClient`: RPC communication.
    - `Database`: PostgreSQL interaction via `sqlx`.
    - `WalletManager`: Keypair management and rotation (Phase 3).
    - `PriceAggregator`: Fetching prices from external sources (Binance, Coingecko).
    - `ApiServer`: Axum-based REST API for observability and control.

### 2. Market Data Service
- **Websocket Ingestion**: `MarketDataService` listens to Solana RPC WebSockets for real-time account updates (OpenBook Slab data).
- **Price Normalization**: Converts raw DEX data into internal price ticks stored in the database.

### 3. Logic & Strategy Layer
- **Pivot Engine**: Dynamic VWAP calculation incorporating market historical data and real-time ticks.
- **Grid Builder**: Asymmetric grid generation logic with exponential volume distribution.
- **Trading Service**: The main orchestrator that runs the localized trading loop, manages state, and coordinates with other services.

### 4. Execution Layer (Jito-First)
- **Bundle Composition**: `OpenBookDex` (infra) handles raw OpenBook instruction building.
- **MEV Protection**: Transactions are bundled and submitted via Jito's Block Engine to prevent front-running and ensure atomicity.

### 5. Safety & Risk Controls
- **Circuit Breaker**: (Planned) Halts trading on excessive drawdown.
- **Graceful Shutdown**: Handles OS signals to cancel all open orders and close connections cleanly.
- **Secret Masking**: Environment variables and masked logging for private keys.

## Design Patterns
- **Orchestrator Pattern**: `TradingService` coordinates the overall flow.
- **Service Pattern**: Domain logic is encapsulated in dedicated services (Pivot, Grid, MarketData).
- **Singleton-like Infrastructure**: Core infrastructure components (`Database`, `SolanaClient`) are shared via `Arc`.
- **Event-Driven**: WebSocket-based market data ingestion triggers updates.

### 6. Resource Management
- **PostgreSQL Storage**: Efficiently stores high-frequency price history and trade logs.
- **Memory Efficiency**: Minimal allocation in the hot path of the trading loop.
