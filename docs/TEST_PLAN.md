# Test Plan - BMV Eco System v2.7

## 1. Overview
This document outlines the testing strategy for the BMV Market Making Bot, focusing on the v2.7 core modules: Financial Manager, Flash Volume, and Rent Recovery.

## 2. Test Objectives
- Ensure accuracy of OpenBook V2 math.
- Verify atomic execution of Jito bundles.
- Validate dynamic rebalancing logic in Financial Manager.
- Confirm browser-based observability is functional.
- Maintain high performance under high trade volume.

## 3. Testing Levels

### 3.1 Unit Testing
- **Location**: `src/` (inline modules).
- **Scope**: Internal logic, math formulas, data parsing.
- **Tools**: `cargo test`.
- **Key Focus**: `openbook.rs` parsing, `financial_manager.rs` ratio logic.

### 3.2 Integration Testing
- **Location**: `tests/integration/`.
- **Scope**: Multi-service coordination using `MockSolanaProvider`.
- **Tools**: `cargo test --test integration_*`.
- **Key Focus**: `TradingService` -> `FinancialManager` -> `SolanaProvider`.

### 3.3 Performance Testing (Benchmarking)
- **Location**: `tests/performance/`.
- **Scope**: Throughput, latency of computation.
- **Tools**: `cargo test --test performance_*`.
- **Key Focus**: Pivot computation with 10k+ trades.

### 3.4 Browser Automation Testing
- **Location**: `tests/browser/`.
- **Scope**: Observability dashboard verification, Grafana/Prometheus health check.
- **Tools**: Playwright (JS/TS) or `fantoccini` (Rust).
- **Key Focus**: Real-time metric updates, TUI/UI data consistency.

## 4. Test Environment
- **Local**: Mocked Solana/DB.
- **CI/CD**: Github Actions / Regxa-staging.
- **Prod**: Mainnet-beta (Dry-run mode).

## 5. Execution Plan
1. Run unit tests on every commit.
2. Run integration tests on PR.
3. Monthly performance audit.
4. Browser smoke tests after UI deployment.
