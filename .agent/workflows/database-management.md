---
description: Database management and migrations
---

Manage PostgreSQL database for the BMV system:

## Check Database Status
```powershell
docker exec postgres-db psql -U botuser -d solana_dex -c "\dt"
```

## View Wallet Data
```powershell
docker exec postgres-db psql -U botuser -d solana_dex -c "SELECT id, pubkey, is_active FROM trading_wallets;"
```

## View Price History
```powershell
docker exec postgres-db psql -U botuser -d solana_dex -c "SELECT * FROM price_history ORDER BY timestamp DESC LIMIT 10;"
```

## View Latency Metrics
```powershell
docker exec postgres-db psql -U botuser -d solana_dex -c "SELECT * FROM latency_ticks ORDER BY timestamp DESC LIMIT 10;"
```

## Reset Database (CAUTION)
```powershell
docker compose down -v
docker compose up -d db
```

## Backup Database
```powershell
docker exec postgres-db pg_dump -U botuser solana_dex > temp/db_backup.sql
```
