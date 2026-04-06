# PhenoObservability

**Comprehensive observability infrastructure for Phenotype - metrics, tracing, logging, and alerting.**

A monorepo containing multiple observability crates with implementations in Rust and Python.

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

## License

MIT OR Apache-2.0
