# Testing Guide

This document outlines the testing strategy and instructions for the Solana Dex (BMV) project (Rust implementation).

## Test Structure

The tests are organized into the following categories:

- **Unit Tests**: Defined within the module files in `src/`.
- **Integration Tests**: Verification of the interaction between multiple modules.
- **Doc Tests**: Verified examples in documentation comments.

## Running Tests

### All Tests

Run all tests including unit, integration, and doc tests:

```powershell
cargo test
```

### Specific Module Tests

To run tests for a specific module:

```powershell
cargo test <module_name>
```

### Integration Tests

If you have dedicated integration tests in the `tests/` directory (Rust style, e.g., `tests/*.rs`):

```powershell
cargo test --test <test_name>
```

## Running with Logging

To see logs during test execution:

```powershell
$env:RUST_LOG="info"; cargo test -- --nocapture
```

## Code Quality

We use `cargo fmt` and `cargo clippy`:

```powershell
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

## Environment Requirements

Some integration tests may require valid RPC URLs in your `.env` file or exported as environment variables:

- `SOLANA_RPC_HTTP_URL`
- `SOLANA_RPC_WS_URL`
