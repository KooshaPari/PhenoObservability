# Traceability Matrix

Real-but-minimal mapping from functional requirements to source and test artifacts.

| Requirement | Source file | Test | Status |
|-------------|-------------|------|--------|
| Rust main entry point | `src/main.rs` | `tests/smoke_test.rs` | ✅ `test_main_runs_without_panic` |
| Go log context propagation | `logctx/logctx.go` | `logctx/logctx_test.go` | ✅ unit tests present |
| Health check endpoint | `health/checker.go` | `tests/smoke_test.rs` | 🚧 smoke-tested only |
| Metrics collection | `metrics/collector.go` | `tests/smoke_test.rs` | 🚧 smoke-tested only |
| OpenTelemetry tracing | `tracing/otel.go` | `tests/smoke_test.rs` | 🚧 smoke-tested only |
| Alerting rules & thresholds | `alerting/rules.go` | `tests/smoke_test.rs` | 🚧 smoke-tested only |
| Structured logging | `logging/structured.go` | `tests/smoke_test.rs` | 🚧 smoke-tested only |
| TypeScript telemetry ports | `ports/telemetry.ts` | `ports/tests/telemetry.test.ts` | ✅ adapter tests present |
