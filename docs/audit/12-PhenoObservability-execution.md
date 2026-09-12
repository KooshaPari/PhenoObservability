# PhenoObservability execution lane

Date: 2026-09-08
Baseline: `1b93793348224797968af2ff9afc9f04c9009bba`
Repair commit: `927400b7`
PR: #249 (draft)

## Root-cause evidence

The baseline branch failed deterministically with `cargo test -p
phenotype-sentinel --no-fail-fast`:

- `phenotype-sentinel` imported `thiserror` in `lib.rs`, `bulkhead.rs`, and
  `rate_limiter.rs` without declaring a direct dependency.
- `circuit_breaker.rs` called `pheno_otel::metrics::record_error`, but the
  `pheno-otel` crate exposed no `metrics` module.
- After restoring those dependencies, stale tests still constructed removed
  `CircuitBreakerError::{Open,HalfOpen}` variants. The current alias is
  `phenotype_errors::DomainError`, whose matching variant is `Validation`.

The first two defects originate in historical commit `4a24448a`, which added
the imports and calls without a complete compile gate.

## Repair and evidence

- Added `thiserror = { workspace = true }` to `phenotype-sentinel`.
- Added a small tracing-backed `pheno_otel::metrics::record_error` API and its
  direct `tracing` dependency.
- Updated the stale tests to the current `DomainError::Validation` contract.
- Focused tests: 75 unit tests and 1 doctest passed.
- Affected-crate check and clippy with `-D warnings`: passed.
- Workspace library tests: passed.
- Full repository format check remains blocked by two pre-existing formatting
  differences in `crates/tracely-core/src/alerting.rs`.
- Pre-commit secret scan is independently blocked by a gitleaks regexp panic
  while parsing `*.lock`; the narrow patch was committed after the checks
  above and this hook failure was recorded.

## Custody

The baseline was preserved before editing:

`/Users/kooshapari/CodeProjects/Phenotype/preservation-evidence/20260908T0726Z/PhenoObservability/PhenoObservability.bundle`

Bundle SHA-256:
`6b0d196e6d5e86805a339e16ed4bbc94e5f5b77413bd5e73a4e9eae82c2d0c7e`

Implementation occurred only in the isolated worktree
`PhenoObservability-wtrees/sentinel-build-repair`. No default branch, reset,
clean, deletion, archive, or merge operation was performed.

## Hosted follow-up at `471be995`

Successful hosted checks include Rust, Unit Tests, Lint & Format, Coverage,
SonarCloud, module-deps, and dependency review. The hosted Cargo Deny and
Security Scan failures were reproducible locally as two patched advisories:
`crossbeam-epoch 0.9.18 -> 0.9.20` and `h2 0.4.15 -> 0.4.16`; `event-listener`
was also advanced from `5.4.1 -> 5.4.2` to clear its unsoundness warning.

The remaining Cargo Deny license failure is a pre-existing transitive
dependency: `xxhash-rust 0.8.15` (BSL-1.0) through `redis -> pheno-dragonfly`.
BSL-1.0 was not added to the allowlist because that would broaden licensing
authority beyond this repair lane. The advisory evidence is from hosted run
`34203181985`, job `101986518091`; the security scan also reported the same
two advisories in run `34203181877`.

The gitleaks failure was a repository configuration defect: `.gitleaks.toml`
used the invalid regexp `*.lock`. It is now `.*\\.lock$`, and local
`gitleaks protect --config .gitleaks.toml --no-banner --redact` passes.
