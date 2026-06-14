# Traceability Matrix

Minimal requirement-to-artifact traceability for the top 5 PhenoObservability features.

| Requirement | Source | Test | Status |
|---|---|---|---|
| FR-MET-001: Metrics Platform (Prometheus-compatible collection, aggregation, dashboards) | `metrics/`, `crates/pheno-questdb/`, `crates/phenotype-surrealdb/` | `tests/smoke_test.rs` (harness scaffolding); crate-level unit tests pending | Partial |
| FR-LOG-001: Log Management (structured JSON logging, aggregation, retention) | `logging/`, `crates/helix-logging/`, `crates/logkit/`, `logctx/` | `tests/smoke_test.rs` (harness scaffolding); crate-level unit tests pending | In Progress |
| FR-TRACE-001: Distributed Tracing (OpenTelemetry, span analysis, sampling) | `tracing/`, `crates/phenotype-observably-tracing/`, `crates/tracely-core/` | `tests/smoke_test.rs` (harness scaffolding); crate-level unit tests pending | In Progress |
| FR-HEALTH-001: Health Monitoring (health check endpoints, SLO/SLI tracking) | `health/`, `src/health.rs` (if present in root src) | `tests/smoke_test.rs` (harness scaffolding); integration tests pending | Partial |
| FR-ALERT-001: Intelligent Alerting (multi-channel alerts, correlation, escalation) | `alerting/`, `crates/phenotype-observably-sentinel/`, `crates/tracely-sentinel/` | `tests/smoke_test.rs` (harness scaffolding); crate-level unit tests pending | Partial |

> **Note:** The repository is actively consolidating LogContext, Severity, RateLimiter, and tracing initialization (see `README.md` Work State). Dedicated per-crate test suites are expected to land as the consolidation completes.
