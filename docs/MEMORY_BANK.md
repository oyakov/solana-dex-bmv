# Memory Bank: BMV Eco System Market Making Bot

The Memory Bank is the central source of truth for the BMV Market Making Bot project. It provides context, architecture patterns, and progress tracking to ensure consistency and speed in development.

## Core Reference Documents
- [**Project Brief**](memory-bank/projectBrief.md): High-level mission and strategy.
- [**Product Context**](memory-bank/productContext.md): Business logic and the problems being solved.
- [**System Patterns**](memory-bank/systemPatterns.md): Architecture, design patterns, and technical decisions.
- [**Tech Context**](memory-bank/techContext.md): Tech stack, libraries, and infrastructure.

## Runtime Status
- [**Active Context**](memory-bank/activeContext.md): Current focus, recent changes, and immediate tasks.
  - Current Version: v0.4.5 (Token Holders Tab)
- [**Progress**](memory-bank/progress.md): Roadmap status and phase completion tracking.
- **Network Observability**: In-system latency tracking for external dependencies (OpenBook, Jito, Solana).

## Operational Guides
- [README](../README.md): Quick start and execution guide.
- [Docker Guide](DOCKER.md): Running the bot in containerized environments.
- [Testing Guide](TESTING.md): Testing strategy and execution instructions.
- [Agent Workflows](WORKFLOWS.md): Available AI agent workflows.
- [Order Grid Visualization](ORDER_GRID_VISUALIZATION.md): Theory and performance monitoring.

## Technical Documentation
- [Main Requirements](requirements/BMV%20Eco%20System%20Market%20Making%20Bot%20—%20Требования%20—%202.7.md): Latest project requirements (v2.7).
- [Technical Spec](customer/Technical%20Spec.md): Detailed technical specification from the customer.
- [Trading Algorithm](requirements/old/Алгоритм%20динамического%20построения%20торговой%20сетки%20ордеров.md): Mathematical details of the grid builder.

## Visual Assets & Diagrams
- [Structural Architecture (Text)](requirements/диаграмма-структурная.txt): Text-based architecture diagram.
- [Structural Architecture (Image)](requirements/диаграмма-структурная.png)
- [General Concept (Image)](requirements/диаграмма-общий-концепт.png)

## Infrastructure & Deployment
- [Lab Data (Regxa)](deploy/Regxa2core2gig/lab-data.md): Deployment-specific data and labs.

## Archive & Legacy
- [Legacy Requirements Folder](requirements/old/): Contains older versions (v2.4 to v2.6) and original draft documents.

## Core Practices
1. **Jito-First**: Every transaction must be MEV-protected via Jito Bundles.
2. **Security-First**: Every engagement is preceded by a **RugCheck** scan.
3. **Financial Autonomy**: The **FinancialManager** handles SOL auto-injection across the swarm.
4. **Stealth**: Always use **Wallet Rotation** and randomized execution delays.
5. **Tokio-Async**: All I/O operations must be asynchronous using the `tokio` runtime.
6. **Hardening**: Mandatory environment variables for secrets, masked configs, and no hardcoded fallbacks for production credentials.
7. **Structured Tracing**: Use the `tracing` crate for all events.
8. **RPC Timeouts**: All RPC calls must have timeouts (500ms for balance queries, 2s for orderbook/token metrics) to prevent API blocking.
9. **Polling Discipline**: Frontend polling intervals: 5s for main dashboard, 30s for latency page. Backend health checks: 300s.
10. **Clippy Compliance**: Code must pass `cargo clippy -- -D warnings` before commit. Use `cargo fmt` for consistent formatting.

