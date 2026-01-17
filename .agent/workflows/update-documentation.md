---
description: Update documentation across the project
---

Update project documentation after implementing changes:

1. Review recent changes
```powershell
git log --oneline -10
```

2. Check which files were modified
```powershell
git diff HEAD~5 --stat
```

3. Update MEMORY_BANK.md with any new patterns or practices
   - Path: `docs/MEMORY_BANK.md`
   - Add new architectural decisions
   - Update version references

4. Update feature documentation if new features were added
   - Check `docs/` folder for relevant files
   - Create new feature docs if needed

5. Update README.md if necessary
   - Installation instructions
   - New environment variables
   - New commands

6. Update version in Cargo.toml if releasing
```powershell
Get-Content Cargo.toml | Select-String -Pattern "version"
```

7. Sync changes with Linear project if applicable
   - Use `/linear-sync-new-work` workflow

8. Commit documentation updates
```powershell
git add docs/; git commit -m "docs: Update documentation"
```
