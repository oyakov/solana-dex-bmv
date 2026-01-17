---
description: Commit changes with proper formatting and checks
---

Pre-commit workflow to ensure code quality:

1. Run code formatting
```powershell
cargo fmt
```

2. Run clippy and fix any warnings
```powershell
cargo clippy -- -D warnings
```

3. Check what files have changed
```powershell
git status --short
```

4. Review changes and stage files
```powershell
git add -A
```

5. Generate a commit message following conventional commits format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `chore:` for maintenance
   - `docs:` for documentation
   - `style:` for formatting
   - `refactor:` for code restructuring

6. Commit with descriptive message

7. Push to origin
```powershell
git push origin main
```
