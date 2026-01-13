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

### 5. Financial & Risk Management
- **Financial Manager**: (New in v0.4.0) Automates SOL auto-injection, BMV rebalancing, and rent recovery across the swarm.
- **RugCheck Integration**: Automated security scanning of tokens before trading to prevent engagement with malicious contracts.
- **Circuit Breaker**: Halts trading on excessive drawdown or critical RPC failures.
- **Graceful Shutdown**: Handles OS signals to cancel all open orders and close connections cleanly.

### 6. Swarm Orchestration
- **Grid Segmentation**: Distributes the trading grid across multiple sub-wallets to bypass the 32-order limit per wallet/market.
- **Wallet Rotation**: Periodic rotation of active sub-wallets to maintain stealth and minimize on-chain footprint.

## Design Patterns
- **Orchestrator Pattern**: `TradingService` coordinates the overall flow.
- **Manager Pattern**: specialized managers (`FinancialManager`, `WalletManager`) handle resource-specific logic.
- **Service Pattern**: Domain logic is encapsulated in dedicated services.
- **Singleton-like Infrastructure**: Core infrastructure components (`Database`, `SolanaClient`) are shared via `Arc`.
- **Event-Driven**: WebSocket-based market data ingestion triggers updates.

### 7. Resource Management
- **PostgreSQL Storage**: Efficiently stores high-frequency price history and trade logs.
- **Dynamic SOL Injection**: Ensures all sub-wallets have sufficient rent/fees for operation.

