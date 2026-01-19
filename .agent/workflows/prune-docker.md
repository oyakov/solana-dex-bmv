---
description: prune docker resources and system-wide cleanup
---
// turbo-all

Prune Docker system, images, volumes, and builder cache to reclaim disk space and RAM.

1. Terminate any lingering powershell/node zombie processes
```powershell
Get-Process -Name "node" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "playwright*" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "pwsh", "powershell" | Where-Object { $_.Id -ne $PID } | Stop-Process -Force
```

2. Prune all unused Docker objects (containers, networks, images)
```powershell
docker system prune -a -f
```

3. Prune unused Docker volumes
```powershell
docker volume prune -f
```

4. Prune unused Docker networks
```powershell
docker network prune -f
```

5. Prune Docker build cache
```powershell
docker builder prune -a -f
```

6. Refresh WSL2 (Optional - recommended if vmmem is still high)
Note: This requires a manual restart of Docker Desktop after completion.
```powershell
# wsl --shutdown
```
