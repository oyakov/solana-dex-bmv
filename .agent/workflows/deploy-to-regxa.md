---
description: Deploy to remote Regxa server
---

Deploy the BMV system to the Regxa-2core2gigs server:

1. Verify local Docker build works
```powershell
docker compose build bot dashboard
```

2. SSH into the server
```
ssh root@146.103.42.174
```

3. Navigate to project directory
```bash
cd /opt/solana-dex-bmv
```

4. Pull latest changes
```bash
git pull origin main
```

5. Rebuild and restart containers
```bash
docker compose down
docker compose build --no-cache
docker compose up -d
```

6. Verify services are running
```bash
docker compose ps
curl http://localhost/health
```

7. Check logs for errors
```bash
docker compose logs --tail 100
```

8. Test dashboard access at http://146.103.42.174

9. Report deployment status
