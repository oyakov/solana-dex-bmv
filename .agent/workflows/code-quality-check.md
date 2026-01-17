---
description: Run code quality checks (fmt, clippy, tests)
---
// turbo-all

Run comprehensive code quality checks before committing:

1. Check code formatting
```powershell
cargo fmt --check
```

2. If formatting fails, apply fixes
```powershell
cargo fmt
```

3. Run clippy with warnings as errors
```powershell
cargo clippy -- -D warnings
```

4. Fix any clippy warnings found

5. Run unit tests
```powershell
cargo test
```

6. Check for compilation errors in release mode
```powershell
cargo check --release
```

7. Report results and any issues that need manual attention
