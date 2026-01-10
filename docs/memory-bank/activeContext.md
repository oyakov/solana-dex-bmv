# Active Context: BMV Eco System Market Making Bot

## Current Focus
Developing the memory bank and ensuring documentation is up to date with the latest technical specifications and requirement documents.

## Recent Changes
- **Project Structure**: Consolidated documentation into `/docs`.
- **Memory Bank**: Initialized core references and practices.
- **Requirements**: Refined the trading grid algorithm (VWAP Pivot, Asymmetric Channel).
- **Trading Orchestrator**: Implemented `TradingService` to manage the main loop and decouple logic from `main.rs`.
- **Observability**: Implemented Prometheus metrics exporter and structured tracing.
- **Health Checks**: Implemented `HealthChecker` for automated connectivity and system verification.
- **Security Hardening**: Implemented environment variable overrides, non-root Docker execution, and secret masking in logs.
- **Pure Rust Migration**: Successfully moved from Python to a 100% Rust implementation.

## In Progress
- [x] Consolidate technical specs.
- [x] Initialize Memory Bank.
- [x] Implement core trading loop orchestrator (Rust).
- [x] Implement Prometheus/Grafana observability stack.
- [x] Implement basic Market Data polling.
- [ ] Integration testing with Jito (In testing).
- [ ] Detailed PnL calculation and reporting.

## Known Issues / Tasks
- Need to verify the exact OpenBook V2 integration details vs legacy OpenBook.
- Coordinate with the team on the final list of $N$ wallet public keys.
- Ensure `config.yaml` is fully synchronized with the `Technical Spec.md`.
