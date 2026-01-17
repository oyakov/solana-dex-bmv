---
description: stop powershell zombie processes
---
// turbo-all

Kill zombie Node.js and PowerShell processes from previous runs:

1. Find and kill any stuck Node.js processes
```powershell
Get-Process -Name "node" -ErrorAction SilentlyContinue | Stop-Process -Force
```

2. Find Playwright processes
```powershell
Get-Process -Name "playwright*" -ErrorAction SilentlyContinue | Stop-Process -Force
```

3. Verify processes are stopped
```powershell
Get-Process -Name "node" -ErrorAction SilentlyContinue
```

4. Report cleanup status
