# Research

## Repository State

- Canonical checkout: `PhenoObservability`
- Canonical HEAD: `a34c5a8`
- Canonical status before work: detached HEAD with unrelated `crates/tracely-sentinel/deny.toml` modification.
- Isolated worktree: `PhenoObservability-wtrees/sladge-current`

## Badge Evidence

- The existing projects-landing ledger referenced older prepared commit `604a0a6` from `PhenoObservability-wtrees/phenobs-sladge-badge`.
- Current detached HEAD did not include the root README Sladge badge before this session.
- README has an existing badge block for license, CI, and Rust, so the Sladge badge belongs there.

## Local Guidance

- `AGENTS.md` requires feature work in `repos/PhenoObservability-wtrees/<topic>/`.
- `CLAUDE.md` lists Rust quality gates: `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, and `cargo test --workspace`.
