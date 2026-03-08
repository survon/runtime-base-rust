# Development Notes

## Git Workflow

**NEVER push to master/main directly.** 

Use feature branches:
```bash
git checkout -b feature/my-feature
# work work work
git push -u origin feature/my-feature
# Create PR on GitHub
```

## Current Repos
- survon-council-seat: https://github.com/SeanCannon/survon-council-seat
- survon-os: https://github.com/survon/survon-os
- runtime-base-rust: https://github.com/survon/runtime-base-rust

## Key Files
- Council UI: src/ui/screens/council/mod.rs
- Council logic: src/module/strategies/council.rs
- survon-os menu: survon-os/scripts/survon.sh

## Testing
- Run council tests: `cargo test council`
