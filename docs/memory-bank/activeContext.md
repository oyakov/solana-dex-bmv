# Active Context: BMV Eco System Market Making Bot

## Current Focus
Maintaining stability of v0.4.8 features and preparing for long-term multi-wallet observability.

## Recent Changes
- **Version 0.4.8 (Current)**:
    - **Backlog Implementation**: Completed all high-priority v2.7 requirements.
    - **SOL Auto-injection**: Automated SOL balance management for trading wallets (BMV-53).
    - **Proximity-based Rebalance**: Intelligent pivot rebalancing based on orderbook proximity (BMV-51).
    - **Swarm Grid Segmentation**: Distributed grid across multiple wallets with 32-order limit enforcement (BMV-50).
    - **L2 Orderbook Scan**: Real-time front-running protection and depth monitoring (BMV-52).
    - **Dynamic Wallet Rotation**: Stealth rotation logic to minimize wallet detection (BMV-46).
    - **RugCheck Monitoring**: Integration for automated security scanning of assets (BMV-14).
    - **Advanced Market Metrics**: Real-time dashboard upgrades with D3 charts for L2 depth and imbalance indices.
    - **Stability Fixes**: Resolved critical compilation errors and "502 Bad Gateway" in Simulation Lab.
- **Version 0.4.5**:
    - **Configuration Profiles**: Split monolithic `config.yaml` into `config/base.yaml`, `config/local.yaml`, and `config/prod.yaml`.
    - **Profile Switching**: Implemented `APP_ENV` based configuration loading with hierarchical merging.
- **Version 0.4.4**:
    - **Dashboard Performance**: Reduced polling intervals (1s→5s main, 10s→30s latency) to reduce RPC load.
    - **API Optimization**: Added 500ms timeouts for wallet balance queries, 2s for orderbook/token metrics.
    - **Docker Stack Optimization**: Removed Grafana to reduce RAM/CPU footprint.
    - **Resource Limits**: Applied CPU (0.5-1.0) and RAM (256MB-512MB) limits to all services.
    - **Docker Profiles**: Introduced `prod` profile for monitoring (Prometheus) and light default dev profile.
    - **Postgres Tuning**: Optimized shared buffers and connections for low-resource environments.
    - **Nginx Integration**: Added nginx/nginx.conf for Docker Compose, separate from deployment config.
    - **Simulation Service**: New backtesting/simulation engine for scenario visualization.
    - **Sidebar Component**: Refactored dashboard navigation into reusable component.
    - **Wallet Management**: Database-backed wallet storage with add/list API endpoints.
    - **CI Compliance**: Fixed all clippy warnings and formatting issues for GitHub Actions.
- **Version 0.4.3**:
    - [x] **Security Hardening**: Non-root Docker, Env var secrets, Masking.
- [x] **Configuration Refactoring**: Profile-based splitting (`local`/`prod`).
 and environment variable enforcement.
- **Version 0.4.1**:
    - **Backlog Implementation**: Completed all high-priority v2.7 requirements.
    - **SOL Auto-injection**: Automated SOL balance management for trading wallets (BMV-53).
    - **Proximity-based Rebalance**: Intelligent pivot rebalancing based on orderbook proximity (BMV-51).
    - **Swarm Grid Segmentation**: Distributed grid across multiple wallets with 32-order limit enforcement (BMV-50).
    - **L2 Orderbook Scan**: Real-time front-running protection and depth monitoring (BMV-52).
    - **Dynamic Wallet Rotation**: Stealth rotation logic to minimize wallet detection (BMV-46).
    - **RugCheck Monitoring**: Integration for automated security scanning of assets (BMV-14).
    - **Advanced Market Metrics**: Real-time dashboard upgrades with D3 charts for L2 depth and imbalance indices.
- **Version 0.3.5**:
    - **Compliance Audit (v2.7)**: Completed comprehensive compliance audit.
- **Version 0.3.4**:
    - **Target Control**: Implemented `target_control_percent` logic.

## In Progress
- [ ] Multi-wallet PnL view consolidation on the dashboard.
- [ ] Jito tip dynamics optimization versus network congestion.
- [ ] Fine-tuning swarm orchestration for maximum stealth.

## Known Issues / Tasks
- [ ] Implement enhanced PnL tracking (BMV-31).
- [ ] Growth Model for profit reinvestment (BMV-32).
- [ ] Review Jito tip dynamics versus network congestion.

