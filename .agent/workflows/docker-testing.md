---
description: Run Docker stack and test all components
---
// turbo-all

Run the complete Docker testing workflow:

1. Stop any existing containers
```powershell
docker compose down
```

2. Build all Docker images (bot, dashboard, nginx)
```powershell
docker compose build
```

3. Start the full stack
```powershell
docker compose up -d
```

4. Wait for services to be healthy (10 seconds)
```powershell
Start-Sleep -Seconds 10
```

5. Check container status
```powershell
docker compose ps
```

6. Test health endpoint
```powershell
curl http://localhost/health
```

7. Test API login
```powershell
echo '{"password":"admin123"}' | docker exec -i bmv-nginx sh -c "curl -s http://bot:8080/api/login -X POST -H 'Content-Type: application/json' -d @-"
```

8. Check bot logs for errors
```powershell
docker compose logs bot --tail 20
```

9. Report the status of all services