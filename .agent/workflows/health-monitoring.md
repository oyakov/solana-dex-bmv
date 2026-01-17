---
description: Check system health and monitoring
---

Monitor the health and status of the BMV system:

## Quick Health Check
1. Check all service endpoints
```powershell
curl http://localhost/health
```

2. Check container status
```powershell
docker compose ps
```

3. Check for errors in logs
```powershell
docker compose logs --tail 30 2>&1 | Select-String -Pattern "error|panic|failed|warn" -CaseSensitive:$false
```

## Detailed Monitoring

4. Check bot resource usage
```powershell
docker stats --no-stream
```

5. Check database connections
```powershell
docker exec postgres-db psql -U botuser -d solana_dex -c "SELECT count(*) FROM pg_stat_activity;"
```

6. Check latency metrics from dashboard
- Navigate to http://localhost/latency
- Review service latencies for Database, Jito, OpenBook, Solana RPC

7. Check wallet balances via API
```powershell
docker exec bmv-nginx curl -s http://bot:8080/api/stats | jq '.total_sol_balance, .total_usdc_balance'
```

8. Generate health report with findings
