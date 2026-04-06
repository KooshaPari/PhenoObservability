# AGENTS.md — PhenoObservability

## Project Overview

- **Name**: PhenoObservability
- **Description**: Comprehensive observability stack - metrics, logging, tracing, health, dashboards, and alerting
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/PhenoObservability`
- **Language Stack**: Rust (core), Go, TypeScript
- **Published**: Internal (Phenotype ecosystem)

## Quick Start Commands

```bash
# Navigate to PhenoObservability
cd /Users/kooshapari/CodeProjects/Phenotype/repos/PhenoObservability

# Rust crates
cargo build --workspace
cargo test --workspace

# Go components
cd go && go build ./...

# Dashboards (if applicable)
cd dashboards && [check README]
```

## Architecture

```
PhenoObservability/
├── ai-prompt-logger/          # AI/LLM prompt logging
├── alerting/                  # Alert management
├── bindings/                  # Language bindings
├── Cargo.lock                # Rust dependencies
├── Cargo.toml                # Workspace manifest
├── crates/                   # Rust observability crates
├── dashboards/               # Grafana/dashboard configs
├── docs/                     # Documentation
├── examples/                 # Usage examples
├── ffi/                      # FFI layer
├── go/                       # Go SDK
├── health/                   # Health check implementations
├── KWatch/                   # Kubernetes watcher
├── logctx/                   # Log context propagation
├── logging/                  # Logging infrastructure
└── ObservabilityKit/         # Unified observability kit
    └── AGENTS.md             # Has own agent rules
```

## Quality Standards

### Rust Components
- **Line length**: 100 characters
- **Formatter**: `cargo fmt`
- **Linter**: `cargo clippy -- -D warnings`
- **Tests**: `cargo test --workspace`

### Go Components
- **Line length**: 100 characters
- **Formatter**: `gofmt`, `goimports`
- **Tests**: `go test ./...`

### Dashboards
- Grafana dashboards must be valid JSON
- Use Grafonnet or similar for generation

## Git Workflow

### Branch Naming
Format: `phenoobs/<type>/<description>` or `<component>/<type>/<description>`

Examples:
- `phenoobs/feat/metrics-v2`
- `logging/fix/rotation-bug`
- `alerting/feat/pagerduty`

### Commit Format
```
<type>(<scope>): <description>

Scope: metrics, logging, tracing, health, alerting, dashboards

Examples:
- feat(metrics): add Prometheus exporter
- fix(logging): resolve rotation deadlock
- chore(alerting): update PagerDuty integration
```

## File Structure

```
PhenoObservability/
├── crates/
│   └── phenotype-metrics/     # Metrics crate
├── go/                        # Go SDK
├── health/                    # Health checks
├── logctx/                    # Log context
├── logging/                   # Logging
├── dashboards/                # Dashboard configs
│   └── *.json                 # Grafana dashboards
├── alerting/                  # Alert configs
└── ObservabilityKit/          # Unified kit
```

## CLI Commands

```bash
# Rust
cargo build --workspace
cargo test --workspace

# Go
cd go && go build ./...

# Dashboard validation
# (if validation tools available)
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Metrics not exporting | Check Prometheus endpoint |
| Logs not correlating | Verify logctx propagation |
| Dashboard errors | Validate JSON structure |
| Health check failures | Check health probe config |

## Dependencies

- **ObservabilityKit**: Unified kit (has own AGENTS.md)
- **crates/phenotype-metrics**: Rust metrics
- **Tracely**: Related tracing project
- **AgilePlus**: Work tracking

## Agent Notes

When working in PhenoObservability:
1. Check `ObservabilityKit/AGENTS.md` for kit-specific rules
2. Multiple subsystems - verify which you're modifying
3. Coordinate with `metrics/` subdirectory for metrics crate
4. Dashboards may be generated - check for generation scripts
