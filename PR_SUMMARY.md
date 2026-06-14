# Merge Prep Summary: `integration/consolidate` → `main`

## What this consolidates

- **CI & Build Hardening** — Fixes for broken submodule entries causing CI failure (`fix/ci-red`, `fix/ci-red2`), OpenSSF Scorecard workflow hardening (disable submodule checkout), Ubuntu runner pinning (`chore/phenoobservability-ubuntu-pin-2026-06-12`), and audit confirming all runners are on free `ubuntu-24.04`/`ubuntu-latest`.
- **Repository Hygiene** — `.gitignore` hardening across Rust, Go, Python, TS, Zig, Mojo, and WASI stacks; `.editorconfig` normalization and verification.
- **Test Coverage Expansion** — Unit tests for `LogEntry::with_field` (logkit), `Level::as_str` / `Display` / `Default` / ordering (domain), and `main()` entry point.
- **Documentation & Traceability** — Traceability matrix skeleton plus populated matrix for the top 5 features (Metrics, Logging, Tracing, Health, Alerting) mapping requirements to source artifacts and test status.
- **Branch Integration** — Consolidates `chore/editorconfig`, `chore/editorconfig2`, `chore/gitignore-hardening`, `ci/free-runners`, `fix/ci-red`, `fix/ci-red2`, `test/onefn3`, and `chore/phenoobservability-ubuntu-pin-2026-06-12` into a single merge target.

## Tests added

| Commit | Test |
|---|---|
| `6b00309` | Unit test for `LogEntry::with_field` |
| `eea0080` | Unit tests for `Level::as_str`, `Display`, `Default`, and ordering |
| `485b5fa` | Unit test for `main()` |

## Traceability

- `docs/traceability.md` — Skeleton matrix for main features.
- `docs/traceability.md` (populated) — Requirement-to-artifact mapping for FR-MET-001, FR-LOG-001, FR-TRACE-001, FR-HEALTH-001, FR-ALERT-001 with source directories and test references.

## Build status

- Workspace `Cargo.toml` resolves cleanly with `resolver = "2"`.
- No new dependencies introduced in this consolidation.
- CI fixes remove broken submodule references that were previously causing red builds.

## Merge risk

**Low** — This branch contains only chore, documentation, test, and CI-fix commits. No API changes, no new production features, and no dependency upgrades. All conflicts in the merge chain were resolved preferring incoming changes. The consolidation is additive (tests + docs) or corrective (CI/repo hygiene).
