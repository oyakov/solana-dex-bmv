---
description: Start lightweight local development environment
---

Start minimal local development stack with reduced resource usage:

## Option A: Database Only (Recommended for Rust development)

// turbo
1. Start only PostgreSQL with minimal resources:
```powershell
docker compose -f docker-compose.local.yml up -d
```

2. Run the bot locally (not in Docker):
```powershell
$env:APP_ENV="local"; cargo run
```

3. Run the dashboard locally with Turbopack:
```powershell
cd dashboard; npm run dev
```

## Option B: Full Local Stack (Docker)

1. Start the local profile stack:
```powershell
docker compose --profile local up -d
```

This starts:
- PostgreSQL (128MB RAM limit)
- Bot (256MB RAM limit)
- Dashboard (512MB RAM limit)

Total: ~900MB RAM vs ~1.8GB for production profile

## Cleanup

// turbo
Stop and remove local containers:
```powershell
docker compose -f docker-compose.local.yml down
```

Or for full local profile:
```powershell
docker compose --profile local down
```

## Resource Comparison

| Service    | Production | Local     |
|------------|------------|-----------|
| PostgreSQL | 256MB      | 128MB     |
| Bot        | 384MB      | 256MB     |
| Dashboard  | 1024MB     | 512MB     |
| Nginx      | 128MB      | N/A       |
| Prometheus | 256MB      | N/A       |
| **Total**  | **2048MB** | **896MB** |
