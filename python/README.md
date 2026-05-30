# Performance Kit

Standalone performance monitoring and analysis toolkit for Python applications. Extracted from the phenoSDK ecosystem for general use.

## Overview

Performance Kit provides a comprehensive set of scripts and tools for:
- **Benchmarking**: Measure execution performance of code, APIs, and systems
- **Profiling**: CPU, memory, and performance profiling
- **Analysis**: Code complexity, dependencies, duplication, and response time analysis
- **Monitoring**: System resource monitoring and test duration tracking

## Installation

### Basic Installation

```bash
pip install -e /Users/kooshapari/CodeProjects/Phenotype/repos/ObservabilityKit/python
```

### With Optional Dependencies

```bash
# Install with all features
pip install -e "/Users/kooshapari/CodeProjects/Phenotype/repos/ObservabilityKit/python[all]"

# Install with specific feature sets
pip install -e "/Users/kooshapari/CodeProjects/Phenotype/repos/ObservabilityKit/python[core,analysis]"
pip install -e "/Users/kooshapari/CodeProjects/Phenotype/repos/ObservabilityKit/python[load,monitoring]"
```

## Script Reference

### Benchmarking Scripts

#### `benchmark.py`
**Purpose**: Comprehensive performance benchmark suite

**What it does**:
- Measures cryptographic operations (MD5, SHA256, OAuth2 encryption)
- Benchmarks storage I/O operations (file read/write)
- Tests memory and conversation operations
- Profiles provider registry operations
- Measures tool execution overhead
- Tests network request latency

**Dependencies**: `aiohttp`, `psutil`, optional internal imports (gracefully degrades)

**Usage**:
```bash
python scripts/benchmark.py
```

**Output**: JSON file with benchmark results, console summary with statistics

---

#### `profiler.py`
**Purpose**: Application performance profiler with optimization recommendations

**What it does**:
- Profiles application startup time
- Measures memory usage via psutil
- Times module imports
- Simulates and profiles tool execution
- Generates optimization recommendations report

**Dependencies**: `psutil`

**Usage**:
```bash
python scripts/profiler.py
```

**Output**: `performance_report.md` with metrics table and recommendations

---

#### `nats_benchmarks.py`
**Purpose**: NATS messaging infrastructure performance benchmarks

**What it does**:
- Benchmarks NATS message publishing throughput
- Measures message latency distribution (p50, p95, p99)
- Tests JetStream persistent messaging performance
- Benchmarks service discovery registration and queries
- Tests metrics recording and retrieval performance
- Runs end-to-end integration benchmarks

**Dependencies**: `asyncio`, optional internal imports (uses mocks when unavailable)

**Usage**:
```bash
python scripts/nats_benchmarks.py [--component {communicator,streaming,discovery,monitoring,integration,all}] [--output FILE]
```

**Output**: JSON results file with detailed metrics

---

### Analysis Scripts

#### `analyze_complexity.py`
**Purpose**: Code complexity analysis using Radon

**What it does**:
- Runs cyclomatic complexity analysis (radon cc)
- Calculates maintainability index (radon mi)
- Computes Halstead complexity metrics (radon hal)
- Calculates overall complexity score and grade
- Generates refactoring recommendations

**Dependencies**: `radon` (external tool)

**Usage**:
```bash
python scripts/analyze_complexity.py [--json] [--report]
```

**Output**: Complexity statistics, grade (A-F), recommendations

---

#### `analyze_dependencies.py`
**Purpose**: Dependency analysis and visualization

**What it does**:
- Analyzes dependency tree with pipdeptree
- Detects dependency conflicts
- Identifies outdated packages
- Calculates dependency depth statistics
- Detects circular dependencies
- Generates security recommendations

**Dependencies**: `pipdeptree`, `pydeps` (external tools)

**Usage**:
```bash
python scripts/analyze_dependencies.py [--json] [--report]
```

**Output**: Dependency statistics, outdated packages list, recommendations

---

#### `analyze_duplication.py`
**Purpose**: Code duplication detection using Pylint

**What it does**:
- Detects duplicate code blocks using pylint
- Groups similar duplicates together
- Calculates duplication percentage
- Generates refactoring recommendations

**Dependencies**: `pylint` (external tool)

**Usage**:
```bash
python scripts/analyze_duplication.py [--json] [--report]
```

**Output**: Duplicate block count, similarity groups, refactoring recommendations

---

#### `analyze_response_times.py`
**Purpose**: API response time measurement and analysis

**What it does**:
- Measures HTTP endpoint response times
- Calculates statistics (mean, median, min, max, std dev)
- Computes percentiles (p50, p90, p95, p99)
- Identifies slow endpoints (>1000ms threshold)
- Tracks failed requests
- Generates comprehensive response time reports

**Dependencies**: `requests`

**Usage**:
```bash
python scripts/analyze_response_times.py [--url URL] [--endpoints ENDPOINTS [ENDPOINTS ...]] [--json] [--report]
```

**Output**: Response time statistics, percentile breakdown, slow endpoint identification

---

#### `analyze_test_structure.py`
**Purpose**: Test file structure analysis

**What it does**:
- Categorizes test files by type (unit, integration, e2e, performance)
- Analyzes directory structure
- Counts test files per category
- Shows test file locations

**Dependencies**: None (standard library only)

**Usage**:
```bash
python scripts/analyze_test_structure.py
```

**Output**: Test file counts by category, directory structure listing

---

### Monitoring Scripts

#### `coverage_analysis.py`
**Purpose**: Test coverage tracking and reporting

**What it does**:
- Runs pytest with coverage collection
- Generates HTML, XML, and JSON coverage reports
- Analyzes coverage trends over time
- Identifies files with low coverage (<80%)
- Tracks coverage against minimum thresholds

**Dependencies**: `pytest`, `pytest-cov`

**Usage**:
```bash
python scripts/coverage_analysis.py [--json] [--report]
```

**Output**: Coverage percentage, low-coverage file list, HTML/JSON/XML reports

---

#### `duration_tracker.py`
**Purpose**: Test duration tracking and slow test identification

**What it does**:
- Tracks individual test execution durations
- Identifies slow tests (>5s threshold, configurable)
- Calculates average and max test duration
- Generates duration reports with recommendations

**Dependencies**: `pytest` (external tool)

**Usage**:
```bash
python scripts/duration_tracker.py [--threshold SECONDS] [--json] [--report]
```

**Output**: Slow test list, duration statistics, optimization recommendations

---

## CLI Entry Points

When installed, the following CLI commands are available:

```bash
# Benchmarking
perfkit-benchmark              # Run comprehensive benchmarks
perfkit-profiler               # Run performance profiler
perfkit-nats-benchmarks        # Run NATS benchmarks (if available)

# Analysis
perfkit-analyze-complexity     # Code complexity analysis
perfkit-analyze-deps           # Dependency analysis
perfkit-analyze-duplication    # Code duplication analysis
perfkit-analyze-response-times # API response time analysis
perfkit-analyze-test-structure # Test structure analysis

# Monitoring
perfkit-coverage               # Coverage analysis
perfkit-duration-tracker       # Test duration tracking
```

## Dependencies Summary

| Script | Core Deps | Optional Deps | External Tools |
|--------|-----------|---------------|----------------|
| benchmark.py | `aiohttp` | phenoSDK internals | - |
| profiler.py | `psutil` | - | - |
| nats_benchmarks.py | - | phenoSDK internals | - |
| analyze_complexity.py | - | - | `radon` |
| analyze_dependencies.py | - | - | `pipdeptree`, `pydeps` |
| analyze_duplication.py | - | - | `pylint` |
| analyze_response_times.py | `requests` | - | - |
| analyze_test_structure.py | - | - | - |
| coverage_analysis.py | - | - | `pytest`, `pytest-cov` |
| duration_tracker.py | - | - | `pytest` |

## Usage Patterns

### CI/CD Integration

```bash
# Run complexity check
python scripts/analyze_complexity.py --report || exit 1

# Check for outdated dependencies
python scripts/analyze_dependencies.py --report

# Run benchmarks
python scripts/benchmark.py

# Check test durations
python scripts/duration_tracker.py --threshold 10.0 --report
```

### Performance Regression Testing

```bash
# Run profiler before changes
python scripts/profiler.py
mv performance_report.md performance_baseline.md

# ... make changes ...

# Run profiler after changes
python scripts/profiler.py
diff performance_baseline.md performance_report.md
```

### API Performance Monitoring

```bash
# Monitor API response times
python scripts/analyze_response_times.py \
    --url http://localhost:8000 \
    --endpoints /health /api/v1/status /metrics \
    --report
```

## Architecture Notes

### Standalone Nature

These scripts were extracted from phenoSDK and are designed to work standalone:
- Optional internal imports are wrapped in try/except blocks
- When phenoSDK internals are unavailable, scripts use mocks or skip related benchmarks
- No hard dependency on pheno.* modules

### Extending the Toolkit

To add new analysis scripts:
1. Create a new script in `scripts/` directory
2. Follow the pattern: `def main(): ... if __name__ == "__main__": raise SystemExit(main())`
3. Add CLI entry point in `pyproject.toml`
4. Document in this README

## License

MIT License - See LICENSE file for details

## Contributing

This package is part of the ObservabilityKit ecosystem. Contributions should follow the project conventions and include appropriate tests.

## Source

- **Original Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/phenoSDK/packages/performance-kit/`
- **Current Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/ObservabilityKit/python/`
- **Extraction Date**: 2026-04-04
- **Status**: Standalone package ready for use
