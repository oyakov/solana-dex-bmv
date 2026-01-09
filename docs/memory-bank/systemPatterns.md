# System Patterns: BMV Eco System Market Making Bot

## Architecture Overview
The system is built as an asynchronous, event-driven trading engine using Python 11.

### 1. Event Loop Orchestrator
- **Central Core**: Owns the `asyncio` loop.
- **Lifecycle Management**: Handles startup, shutdown, and task monitoring.
- **Signal Handling**: Graceful shutdown on termination signals.

### 2. Market Data Pipeline
- **Subscribers**: Listen to Solana RPC WebSockets for account updates (OpenBook, Raydium).
- **Normalizers**: Convert raw program data into standard internal models.
- **Queues**: Push normalized events to the strategy engine.

### 3. Strategy Engine
- **VWAP Pivot Calculation**: Dynamic calculation of the "True Average Price" including all costs (Market ID rent, Jito tips, fees).
- **Asymmetric Grid Builder**: 
    - **Buy Side**: 15% width, exponential volume distribution (larger walls further from pivot).
    - **Sell Side**: 30% width, exponential volume distribution.
- **Wallet Manager**: Rotates actions among $N$ wallets to bypass OpenBook's 32-order limit and provide stealth.

### 4. Execution Layer (Jito-First)
- **Bundle Composition**: Combines multiple instructions (Cancel, Settle, Place) into a single atomic bundle.
- **Priority Submission**: Uses Jito's Block Engine for MEV-protected, high-speed execution.
- **Dry-run Mode**: For safe testing without real funds.

### 5. Safety & Risk Controls
- **Circuit Breaker**: Halts trading if losses exceed a threshold within a time window.
- **Kill Switch**: Immediate halt via a sentinel file or TUI command.
- **Fiat Floor**: Dynamically adjusts the SOL-denominated grid to maintain a minimum USD value.

## Design Patterns
- **Orchestrator Pattern**: Centralized control of data flow.
- **Strategy Pattern**: Decoupling the trading logic from the execution layer.
- **Observer Pattern**: For market data updates.
- **Singleton/Service Registry**: For shared components like Wallet Manager and RPC Clients.

### 6. Resource Management
- **Rent Recovery**: Automatically identifies and closes idle OpenBook/Token accounts to recover SOL rent.
- **Wallet Rebalancing**: Ensures SOL for gas and Jito tips is distributed correctly among active trading wallets.
