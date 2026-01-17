---
description: Full rebuild and test of Docker stack
---
// turbo-all

Complete rebuild and verification workflow:

1. Stop all containers
```powershell
docker compose down -v
```

2. Clean Docker build cache (optional, for fresh build)
```powershell
docker builder prune -f
```

3. Rebuild all images from scratch
```powershell
docker compose build --no-cache bot dashboard
```

4. Start the stack
```powershell
docker compose up -d
```

5. Wait for initialization (15 seconds)
```powershell
Start-Sleep -Seconds 15
```

6. Verify all containers are running
```powershell
docker compose ps
```

7. Test the API endpoints:
   - Health: `curl http://localhost/health`
   - Login: Test via browser or curl
   - Stats: Verify dashboard shows data

8. Check for errors in logs
```powershell
docker compose logs --tail 50 2>&1 | Select-String -Pattern "error|panic|failed" -CaseSensitive:$false
```

9. Open dashboard in browser at http://localhost

10. Report results including any issues found