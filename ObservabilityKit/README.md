# ObservabilityKit

[![Build](https://img.shields.io/github/actions/workflow/status/KooshaPari/ObservabilityKit/ci.yml?branch=main&label=build)](https://github.com/KooshaPari/ObservabilityKit/actions)
[![Release](https://img.shields.io/github/v/release/KooshaPari/ObservabilityKit?include_prereleases&sort=semver)](https://github.com/KooshaPari/ObservabilityKit/releases)
[![License](https://img.shields.io/github/license/KooshaPari/ObservabilityKit)](LICENSE)
[![Phenotype](https://img.shields.io/badge/Phenotype-org-blueviolet)](https://github.com/KooshaPari)


**Unified Observability SDKs & Instrumentation for Phenotype**

ObservabilityKit provides language-specific SDKs and instrumentation libraries for OpenTelemetry integration. It simplifies adding distributed tracing, metrics collection, and structured logging to any service—with automatic context propagation and minimal boilerplate.

## Overview

ObservabilityKit abstracts away OpenTelemetry complexity. Instead of managing spans, metrics, and logs separately, services import a single SDK that provides unified instrumentation. The kit auto-configures exporters, handles context propagation across RPC boundaries, and provides sensible defaults for latency thresholds, sampling, and cardinality controls.

## Technology Stack

- **Languages**: Rust, Python, Go, TypeScript (multi-language)
- **Instrumentation Standard**: OpenTelemetry (OTEL)
- **Core Exporters**:
  - **Traces**: OTLP (gRPC/HTTP) → Tempo
  - **Metrics**: Prometheus scrape endpoint or OTLP
  - **Logs**: OTLP or direct to Loki
- **Integrations**: 
  - Web frameworks (actix, FastAPI, Gin, Express)
  - Database drivers (sqlx, asyncpg, pymongo)
  - Message queues (Kafka, RabbitMQ)
  - HTTP clients (reqwest, httpx, http.Client)
- **Context**: OpenTelemetry Baggage for cross-service context

## Key Features

- **Auto-Instrumentation**: Macro-based span creation with automatic timing and error capture
- **Framework Integration**: Drop-in middleware for web frameworks and database drivers
- **Distributed Tracing**: Automatic parent/child span linking and context propagation
- **Metrics Collection**: Pre-built metrics (latency, error rates, throughput) with custom dimensions
- **Structured Logging**: Log events automatically correlated with trace context
- **Sampling Strategy**: Configurable sampling with head-based and tail-based options
- **Cardinality Controls**: Protect Prometheus from high-cardinality dimensions
- **Local Testing**: In-memory exporters for offline testing without Tempo/Prometheus
- **Performance**: Minimal overhead (<5% latency impact) with async exporters and batching
- **Configuration**: Environment-variable driven setup (OTEL_* standard)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/KooshaPari/ObservabilityKit.git
cd ObservabilityKit

# Review the instrumentation guide
cat docs/INSTRUMENTATION_GUIDE.md

# Example: Rust service with automatic tracing
cat examples/rust-web-service/src/main.rs

# Run example service
cd examples/rust-web-service && cargo run

# View traces in Tempo (requires Tracera running)
curl http://localhost:16686/  # Jaeger UI alternative

# Run tests
cargo test --all --workspace
pytest tests/

# Review multi-language examples
ls examples/
```

## Project Structure

```
ObservabilityKit/
├── rust/
│   ├── observability-core/        # Base Rust SDK
│   │   ├── src/
│   │   │   ├── tracer.rs          # Trace initialization
│   │   │   ├── metrics.rs         # Metrics collection
│   │   │   ├── logger.rs          # Structured logging
│   │   │   └── context.rs         # Context propagation
│   │   └── tests/
│   ├── observability-actix/       # Actix-web middleware
│   ├── observability-sqlx/        # SQLx database instrumentation
│   └── observability-http/        # HTTP client/server tracing
├── python/
│   ├── observability_kit/         # Python SDK
│   │   ├── tracer.py              # OTEL setup
│   │   ├── metrics.py             # Metrics collection
│   │   └── logging.py             # Structured logging
│   ├── observability_fastapi/     # FastAPI middleware
│   ├── observability_sqlalchemy/  # SQLAlchemy instrumentation
│   └── tests/
├── go/
│   ├── observability/             # Go SDK
│   ├── observability/middleware/  # Gin/Echo middleware
│   └── tests/
├── typescript/
│   ├── observability-core/        # TypeScript SDK
│   ├── observability-express/     # Express middleware
│   └── tests/
├── examples/
│   ├── rust-web-service/          # Actix example with tracing
│   ├── python-api/                # FastAPI example
│   ├── go-backend/                # Gin example
│   └── distributed-demo/          # Multi-service tracing
├── docs/
│   ├── INSTRUMENTATION_GUIDE.md   # How to add tracing
│   ├── EXPORTERS.md               # Tempo/Prometheus config
│   ├── CONTEXT_PROPAGATION.md     # Cross-service linking
│   ├── SAMPLING.md                # Sampling strategies
│   └── TROUBLESHOOTING.md         # Common issues
└── tests/
    ├── integration/               # Multi-language tests
    ├── performance/               # Overhead benchmarks
    └── fixtures/                  # Test telemetry data
```

## Related Phenotype Projects

- **Tracera** — Observability platform that consumes ObservabilityKit telemetry
- **PhenoObservability** — Higher-level observability patterns built on ObservabilityKit
- **AgilePlus** — Instrumented with ObservabilityKit for monitoring
- **HexaKit** — Observability port implementations use ObservabilityKit SDKs

## Quality & Testing

All SDKs maintain comprehensive test coverage:
- Unit tests for span/metric creation
- Integration tests with real exporters (testcontainers)
- Performance benchmarks tracking instrumentation overhead
- Example services validating end-to-end tracing

```bash
# Run all tests
cargo test --workspace
pytest tests/ -v
go test ./...

# Benchmark instrumentation overhead
cargo bench --workspace --bench overhead
```

## Instrumentation Standards

All services must implement:
- **Root Span**: One root span per request/message
- **Semantic Attributes**: Standard OTEL attributes (http.method, db.system, etc.)
- **Error Recording**: Automatic error capture in spans
- **Baggage**: Service name, environment, version in all traces

See `INSTRUMENTATION_GUIDE.md` for detailed requirements.

## Governance

All work tracked in AgilePlus. New SDKs or integrations must:
- Support all four languages (Rust/Python/Go/TS)
- Include comprehensive examples
- Document performance impact
- Maintain backward compatibility

## License

MIT — see [LICENSE](./LICENSE).

---

**Version**: v0.1.0  
**Last Updated**: 2026-04-25
