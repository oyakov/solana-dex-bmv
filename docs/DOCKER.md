# Docker Guide

This guide provides instructions for running the Solana Dex (BMV) bot using Docker.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

## Quick Start

1. **Prepare Environment**:

   ```powershell
   copy .env.example .env
   ```

   Edit `.env` and provide your Solana RPC URLs and keypair paths.

2. **Build and Start**:

   ```powershell
   docker-compose up -d --build
   ```

3. **Check Status**:

   ```powershell
   docker-compose ps
   docker-compose logs -f
   ```

## Configuration in Docker

### Volume Mapping

By default, the `docker-compose.yml` maps:

- `./data` -> `/app/data` (for SQLite database persistence)
- `./keys` -> `/app/keys` (for your Solana keypairs)

Ensure your `.env` refers to paths *inside* the container, e.g.:
`WALLET_KEYPAIR_PATH=/app/keys/my-wallet.json`

### Commands

To run the bot with custom arguments:

```powershell
docker run --env-file .env solana-dex-bmv /app/solana-dex-bmv --config /app/config.yaml
```

## Troubleshooting

### Container Crashes

Check the logs for initialization errors:

```powershell
docker-compose logs bot
```

### Networking Issues

If the container cannot reach your local RPC or Jito, ensure your Docker network settings allow outbound traffic. If running a local RPC node on the host, use `host.docker.internal` (Windows/Mac) or the host's IP address.

### Permissions

Ensure the `./data` directory on your host has write permissions for the user running the Docker container.
