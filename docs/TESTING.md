# Testing Guide

This document outlines the testing strategy and instructions for the Solana Dex (BMV) project.

## Test Structure

The tests are organized into three main categories:

- **Unit Tests (`tests/unit/`)**: Verify individual components in isolation (e.g., Grid Builder logic, Pivot Engine calculations).
- **Integration Tests (`tests/integration/`)**: Verify the interaction between multiple modules (e.g., Bot flow, persistence).
- **Smoke/Legacy Tests (`tests/*.py`)**: Basic functionality checks.

## Running Tests

### Basic Unit Tests

Run all unit tests:

```bash
pytest tests/unit
```

### Integration Tests

Run integration tests (requires SQLite and environment setup):

```bash
pytest tests/integration
```

### Devnet Connectivity Test

To verify the bot can connect to Solana Devnet and Jito:

```bash
pytest tests/integration/test_devnet_connectivity.py
```

## Advanced Testing Scenarios

### 1. Persistence & Recovery

Verifies that the bot correctly saves its state to SQLite and can recover after a restart.

```bash
pytest tests/integration/test_persistence_recovery.py
```

### 2. Stress & Limits

Tests the `GridBuilder` and `RiskManager` against extreme market conditions and edge cases.

```bash
pytest tests/unit/test_stress_limits.py
```

### 3. Jito Logic

Verifies the assembly and signing of Jito bundles without actually submitting them (using mocks).

```bash
pytest tests/unit/test_jito_logic.py
```

## Environment Requirements

Some integration tests require valid RPC URLs in your `.env` file or exported as environment variables:

- `SOLANA_RPC_HTTP_URL`
- `SOLANA_RPC_WS_URL`

## Code Quality

We use `ruff` for linting and formatting:

```bash
ruff check .
ruff format .
```
