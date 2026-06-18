# PhenoObservability — Status

**Last updated:** 2026-06-17  
**Disposition:** [`docs/boundary/DISPOSITION.md`](docs/boundary/DISPOSITION.md)  
**Audit:** [`docs/audit/BLOCK-C-AUDIT.md`](docs/audit/BLOCK-C-AUDIT.md)

## Boundary verdict

**AFFIRM / KEEP ACTIVE** — canonical `observe` role workspace; Python facade in SDK.

| Layer | Status | Canonical owner |
|-------|--------|-----------------|
| Rust tracing (`tracingkit`, tracely-*) | Active | **This repo** `crates/` |
| Rust metrics (`metrickit`, phenotype-metrics) | Active | **This repo** |
| Rust logging / telemetry / health | Active | **This repo** `rust/` + `crates/` |
| Wave A inbound (sentry-config) | Pending | **This repo** `rust/` (from HexaKit) |
| Python observability facade | Redirect | `phenotype-python-sdk/packages/observability-kit` |
| OTLP thin init | Pending merge | `phenotype-otel` → this repo |
| G2 chokepoint repoint | Done | `vendor/` → phenoShared (Wave E) |

## Consumer guidance

- **Rust:** git dependency on `KooshaPari/PhenoObservability` `main` (workspace crates) until crates.io publish.
- **Python:** install from [`phenotype-python-sdk`](https://github.com/KooshaPari/phenotype-python-sdk) `packages/observability-kit` — ObservabilityKit subtree removed from this repo (PR #157).

## Next actions

1. Merge Block-C disposition PR.
2. Port `phenotype-sentry-config` (Wave A #35).
3. Merge `phenotype-otel`; decompose misplaced `rust/` crates.
4. phenoShared publish → delete `vendor/`.
