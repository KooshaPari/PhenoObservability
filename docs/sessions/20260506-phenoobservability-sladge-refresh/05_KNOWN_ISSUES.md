# Known Issues

## Pre-existing

- Canonical PhenoObservability is detached and has an unrelated local modification in `crates/tracely-sentinel/deny.toml`.
- Several old worktree entries are prunable; this session does not clean unrelated worktree metadata.

## Session Blockers

- `cargo fmt --check` is blocked during Cargo metadata loading by missing sibling path dependency `PhenoObservability-wtrees/pheno/crates/phenotype-errors/Cargo.toml`.
