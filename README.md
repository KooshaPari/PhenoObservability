# PhenoObservability

Comprehensive observability infrastructure for Phenotype — distributed tracing, metrics, structured logging, and alerting. Built as a Rust + Python monorepo providing pluggable observability backends for the entire Phenotype ecosystem.

## Overview

PhenoObservability provides production-grade observability tooling including OpenTelemetry-based distributed tracing, Prometheus-compatible metrics collection, structured JSON logging, and multi-channel alerting. All components are designed for integration into Phenotype services with minimal overhead. Crates range from low-level instrumentation primitives to high-level dashboard integrations.

## Crates

| Crate | Description |
|-------|-------------|
| **pheno-dragonfly** | Dragonfly cache integration for observability |
| **pheno-questdb** | QuestDB time-series database integration |
| **pheno-tracing** | Distributed tracing with OpenTelemetry |
| **phenotype-llm** | LLM observability and logging |
| **phenotype-mcp-server** | MCP server for observability tools |
| **phenotype-surrealdb** | SurrealDB integration for metrics storage |
| **tracely-core** | Core tracing infrastructure |
| **tracely-sentinel** | Sentinel-based tracing and monitoring |
| **helix-logging** | Structured logging framework |
| **tracingkit** | Comprehensive tracing toolkit |

## Features

### Tracing
- OpenTelemetry integration
- Distributed tracing across services
- Custom span attributes
- Sampling strategies

### Metrics
- Prometheus-compatible metrics
- Time-series storage
- Custom metric collectors
- Alerting integration

### Logging
- Structured JSON logging
- Log sampling
- Context propagation
- Multiple output formats

### Alerting
- Threshold-based alerts
- Rate-of-change alerts
- Composite alerts
- Escalation policies

## Quick Start

```rust
use pheno_tracing::{init_tracing, Span};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    
    let span = Span::new("my_operation");
    span.record("user_id", "123");
    
    // Your code here
    Ok(())
}
```

## Architecture

```
crates/
├── pheno-dragonfly/      # Cache observability
├── pheno-questdb/        # Time-series storage
├── pheno-tracing/        # Tracing infrastructure
├── phenotype-llm/        # LLM observability
├── phenotype-mcp-server/  # MCP integration
├── phenotype-surrealdb/   # Metrics storage
├── tracely-core/         # Core tracing
├── tracely-sentinel/     # Sentinel monitoring
├── helix-logging/        # Structured logging
└── tracingkit/           # Tracing toolkit
```

## Technology Stack

- **Core Language**: Rust (async/await with Tokio)
- **Python Integration**: PyO3 for native bindings
- **Telemetry Standards**: OpenTelemetry, OTEL Protocol (OTLP)
- **Backends**: Jaeger, Datadog, Prometheus, Grafana, SurrealDB, QuestDB
- **Caching**: Dragonfly (Redis-compatible)
- **Serialization**: serde + JSON for structured logs
- **Async Runtime**: Tokio with multi-threaded scheduler

## Key Features

- **Zero Overhead When Disabled**: Compile-time feature flags for performance-critical paths
- **Sampling Strategies**: Deterministic, probabilistic, and custom sampling for cost control
- **Context Propagation**: W3C Trace Context and Jaeger propagation formats
- **Log Correlation**: Automatic span-to-log correlation with trace IDs
- **Custom Metrics**: Histogram, counter, gauge, and distribution collectors
- **Alerting Pipelines**: Threshold-based, anomaly detection, escalation workflows
- **Dashboard Export**: Pre-built Grafana dashboards for common Phenotype services

## Quick Start

```bash
# Clone and enter repo
git clone https://github.com/KooshaPari/PhenoObservability.git
cd PhenoObservability

# Review governance
cat CLAUDE.md

# Build all observability crates
cargo build --all-features
cargo build --release

# Run comprehensive tests
cargo test --workspace

# Generate code coverage report (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --lib --out=Html
```

## Testing

### Run All Tests

```bash
cargo test --workspace
```

### Run Library Tests Only

```bash
cargo test --workspace --lib
```

### Run Tests for Specific Crate

```bash
cargo test -p pheno-tracing
cargo test -p tracely-sentinel
cargo test -p helix-logging
```

### Coverage Target

Maintain **80%+ code coverage** on all crates. See `docs/FUNCTIONAL_REQUIREMENTS.md` for complete test traceability matrix.

## Related Phenotype Projects

- **[Tracera](../Tracera/)** — Distributed request tracing; primary consumer of PhenoObservability
- **[cloud](../cloud/)** — Multi-tenant platform using observability for SLA monitoring
- **[PhenoDevOps](../PhenoDevOps/)** — CI/CD observability and pipeline metrics

## Governance & Contributing

- **CLAUDE.md** — Project conventions, backend integration policies
- **Functional Requirements**: [docs/FUNCTIONAL_REQUIREMENTS.md](docs/FUNCTIONAL_REQUIREMENTS.md)
- **Operator Guide**: [docs/guides/](docs/guides/)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

## License

MIT OR Apache-2.0
