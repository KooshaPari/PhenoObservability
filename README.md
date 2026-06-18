<!-- AI-DD-META:START -->
<!-- This repository is planned, maintained, and managed by AI Agents only. -->
<!-- Slop issues are expected and intentionally present as part of an HITL-less -->
<!-- /minimized AI-DD metaproject of learning, refining, and building brute-force -->
<!-- training for both agents and the human operator. -->
![Downloads](https://img.shields.io/github/downloads/KooshaPari/PhenoObservability/total?style=flat-square&label=downloads&color=blue)
![GitHub release](https://img.shields.io/github/v/release/KooshaPari/PhenoObservability?style=flat-square&label=release)
![License](https://img.shields.io/github/license/KooshaPari/PhenoObservability?style=flat-square)
![AI-Slop](https://img.shields.io/badge/AI--DD-Slop%20Expected-orange?style=flat-square)
![AI-Only-Maintained](https://img.shields.io/badge/Planned%20%26%20Maintained%20by-AI%20Agents%20Only-red?style=flat-square)
![HITL-less](https://img.shields.io/badge/HITL--less%20AI--DD-metaproject-yellow?style=flat-square)

> ⚠️ **AI-Agent-Only Repository**
>
> This repo is **planned, maintained, and managed exclusively by AI Agents**.
> Slop issues, rough edges, and AI artifacts are **expected and intentionally
> present** as part of an **HITL-less / minimized AI-DD** metaproject focused
> on learning, refining, and brute-force training both the agents and the
> human operator. Bug reports and contributions are still welcome, but please
> expect AI-generated code, comments, and documentation throughout.
<!-- AI-DD-META:END -->

> **Boundary disposition (Block-C, 2026-06-17):** This repo is the canonical **`observe` role**
> workspace — Rust tracing/metrics/logging/health live here.
> **Python** consumers → [`phenotype-python-sdk/packages/observability-kit`](https://github.com/KooshaPari/phenotype-python-sdk/tree/main/packages/observability-kit).
> See [`docs/boundary/DISPOSITION.md`](docs/boundary/DISPOSITION.md) · [`BOUNDARY.md`](BOUNDARY.md) · [`STATUS.md`](STATUS.md).

## Work State

| Field | Value |
|---|---|
| Last commit | 2026-06-17 |
| Open issues | 6 |
| Open PRs | 0 |
| Focus | Block-C disposition; Wave A sentry-config port; phenotype-otel merge |

Progress: ██████░░░░ 60%

> **Work state:** ACTIVE · **Progress:** `███████░░░ 65%`
> Rust observability stack (logging/metrics/tracing/health); multi-lang bindings; phantom HexaKit dep to fix · updated 2026-06-02

> **Pinned references (Phenotype-org)**
> - MSRV: see rust-toolchain.toml
> - cargo-deny config: see deny.toml
> - cargo-audit: rustsec/audit-check@v2 weekly
> - Branch protection: 1 reviewer required, no force-push
> - Authority: phenotype-org-governance/SUPERSEDED.md

> **Pinned references (Phenotype-org)**
> - MSRV: see rust-toolchain.toml
> - cargo-deny config: see deny.toml
> - cargo-audit: rustsec/audit-check@v2 weekly
> - Branch protection: 1 reviewer required, no force-push
> - Authority: phenotype-org-governance/SUPERSEDED.md

# PhenoObservability

**Status:** maintenance

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/KooshaPari/PhenoObservability/actions/workflows/ci.yml/badge.svg)](https://github.com/KooshaPari/PhenoObservability/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![AI Slop Inside](https://sladge.net/badge.svg)](https://sladge.net)

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
| **logkit** (Logify subtree) | Hexagonal structured logging SDK (`crates/logkit/`) |
| **tracingkit** | Comprehensive tracing toolkit |
| **metrickit** | Hexagonal metrics (`Metron` absorption) |

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

### Runtime profiling
Shell/Python profiler toolkit migrated from archived [Profila](https://github.com/KooshaPari/Profila) — see [`profiling/`](profiling/README.md) for CPU, memory, I/O, and complexity analysis scripts.

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

## Python observability facade

Python SDK consumers should use [`phenotype-python-sdk/packages/observability-kit`](https://github.com/KooshaPari/phenotype-python-sdk/tree/main/packages/observability-kit). The archived `ObservabilityKit` repo and embedded subtree were removed to eliminate triple-copy maintenance.

## Logify / logkit (subtree)

The [Logify](https://github.com/KooshaPari/Logify) structured-logging SDK is absorbed under `crates/logkit/` (squashed subtree). Build standalone:

```bash
cargo check --manifest-path crates/logkit/Cargo.toml
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
├── metrickit/            # Metrics (Metron absorption)
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

MIT — see [LICENSE](./LICENSE).
