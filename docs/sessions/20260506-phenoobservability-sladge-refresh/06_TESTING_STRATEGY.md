# Testing Strategy

## Focus

This is a root README governance refresh, so validation focuses on:

- Whitespace-safe diff checks.
- README badge presence.
- Available Rust formatting gate for regression awareness.

## Commands

```bash
git diff --check
rg -n "sladge|AI Slop" README.md docs/sessions/20260506-phenoobservability-sladge-refresh
cargo fmt --check
```

## Results

- `git diff --check`: passed.
- Badge presence search: passed.
- `cargo fmt --check`: blocked during Cargo metadata loading by missing sibling path dependency `PhenoObservability-wtrees/pheno/crates/phenotype-errors/Cargo.toml`.
