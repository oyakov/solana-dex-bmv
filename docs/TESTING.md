# Testing Guide - BMV Market Making Bot

This guide explains how to run and extend the testing suite for the BMV bot.

## 1. Unit Tests
Unit tests are located inside the `src/` directory within their respective modules. They test isolated logic and math.

**Run all unit tests:**
```powershell
cargo test --lib
```

## 2. Integration Tests
Integration tests are located in `tests/integration/`. They verify the cooperation between multiple services (e.g., Financial Manager and Solana Client) using mocks.

**Run all integration tests:**
```powershell
cargo test --test integration_*
```

**Specific tests:**
- `rebalance_logic`: Verifies SOL/USDC rebalancing triggers.
- `flash_volume_jito`: Verifies atomic wash trade bundle construction.
- `rent_recovery`: Verifies OpenOrder account scanning.

## 3. Performance Tests (Benchmarks)
Located in `tests/performance/`. Used to measure the latency of critical computations like the Pivot Point calculation.

**Run performance tests:**
```powershell
cargo test --test performance_* -- --nocapture
```

## 4. Browser Automation
Located in `tests/browser/`. Uses Playwright to verify the observability dashboard (Grafana).

**Prerequisites:**
- Node.js installed.
- `npm install playwright`

**Run smoke test:**
```powershell
node tests/browser/dashboard_smoke.js
```

## 5. Adding New Tests
- **Common Helpers**: Use `tests/common/mod.rs` to share setup logic or custom mock expectations.
- **Mocks**: Add new mockable methods to `solana_dex_bmv::infra::mocks` if needed.
