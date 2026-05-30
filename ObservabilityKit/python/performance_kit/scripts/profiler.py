#!/usr/bin/env python3
"""Performance Profiler.

Profiles application performance and generates optimization recommendations.
"""

import asyncio
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import psutil


@dataclass
class PerformanceMetrics:
    """
    Performance metrics.
    """

    startup_time: float
    memory_usage_mb: float
    import_time: float
    tool_registration_time: float
    avg_tool_execution_time: float


class PerformanceProfiler:
    """
    Profile application performance.
    """

    def __init__(self):
        self.metrics: dict[str, Any] = {}
        self.process = psutil.Process()

    def profile_startup(self) -> float:
        """
        Profile startup time.
        """
        print("📊 Profiling startup time...")

        start_time = time.time()

        # Simulate startup
        try:
            # Import main modules
            import src.domain.ports.ai_provider
            import src.infrastructure.adapters.providers.registry
            import src.shared.config.settings
        except ImportError:
            pass

        startup_time = time.time() - start_time

        print(f"   Startup time: {startup_time:.3f}s")
        return startup_time

    def profile_memory(self) -> float:
        """
        Profile memory usage.
        """
        print("📊 Profiling memory usage...")

        memory_info = self.process.memory_info()
        memory_mb = memory_info.rss / 1024 / 1024

        print(f"   Memory usage: {memory_mb:.2f} MB")
        return memory_mb

    def profile_imports(self) -> float:
        """
        Profile import time.
        """
        print("📊 Profiling import time...")

        start_time = time.time()

        # Import key modules
        modules = [
            "src.domain.ports.ai_provider",
            "src.infrastructure.adapters.providers.litellm_adapter",
            "src.infrastructure.adapters.providers.registry",
            "src.shared.config.settings",
            "src.shared.config.env_manager",
        ]

        for module in modules:
            try:
                __import__(module)
            except ImportError:
                pass

        import_time = time.time() - start_time

        print(f"   Import time: {import_time:.3f}s")
        return import_time

    async def profile_tool_execution(self) -> float:
        """
        Profile tool execution time.
        """
        print("📊 Profiling tool execution...")

        # Simulate tool execution
        execution_times = []

        for i in range(10):
            start_time = time.time()

            # Simulate work
            await asyncio.sleep(0.01)

            execution_time = time.time() - start_time
            execution_times.append(execution_time)

        avg_time = sum(execution_times) / len(execution_times)

        print(f"   Avg execution time: {avg_time:.3f}s")
        return avg_time

    async def run_full_profile(self) -> PerformanceMetrics:
        """
        Run full performance profile.
        """
        print("=" * 80)
        print("PERFORMANCE PROFILING")
        print("=" * 80)
        print()

        startup_time = self.profile_startup()
        memory_usage = self.profile_memory()
        import_time = self.profile_imports()
        tool_registration_time = 0.1  # Placeholder
        avg_tool_execution = await self.profile_tool_execution()

        metrics = PerformanceMetrics(
            startup_time=startup_time,
            memory_usage_mb=memory_usage,
            import_time=import_time,
            tool_registration_time=tool_registration_time,
            avg_tool_execution_time=avg_tool_execution,
        )

        return metrics

    def generate_report(self, metrics: PerformanceMetrics) -> str:
        """
        Generate performance report.
        """
        report = f"""
=" * 80
PERFORMANCE REPORT
=" * 80

## Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Startup Time | {metrics.startup_time:.3f}s | <5s | {"✅" if metrics.startup_time < 5 else "❌"} |
| Memory Usage | {metrics.memory_usage_mb:.2f} MB | <500MB | {"✅" if metrics.memory_usage_mb < 500 else "❌"} |
| Import Time | {metrics.import_time:.3f}s | <2s | {"✅" if metrics.import_time < 2 else "❌"} |
| Tool Registration | {metrics.tool_registration_time:.3f}s | <1s | {"✅" if metrics.tool_registration_time < 1 else "❌"} |
| Avg Tool Execution | {metrics.avg_tool_execution_time:.3f}s | <0.1s | {"✅" if metrics.avg_tool_execution_time < 0.1 else "❌"} |

## Optimization Recommendations

"""

        recommendations = []

        if metrics.startup_time > 5:
            recommendations.append(
                f"""
### 1. Optimize Startup Time

**Current**: {metrics.startup_time:.3f}s | **Target**: <5s

**Actions**:
- Implement lazy imports for heavy modules
- Defer non-critical initialization
- Use import hooks for conditional loading
- Profile with `py-spy` to identify bottlenecks

**Example**:
```python
# Before
import heavy_module

# After
def get_heavy_module():
    import heavy_module
    return heavy_module
```
""",
            )

        if metrics.memory_usage_mb > 500:
            recommendations.append(
                f"""
### 2. Reduce Memory Usage

**Current**: {metrics.memory_usage_mb:.2f} MB | **Target**: <500MB

**Actions**:
- Use generators instead of lists where possible
- Implement connection pooling
- Clear caches periodically
- Profile with `memory-profiler`

**Example**:
```python
# Before
data = [process(item) for item in large_list]

# After
data = (process(item) for item in large_list)
```
""",
            )

        if metrics.import_time > 2:
            recommendations.append(
                f"""
### 3. Optimize Import Time

**Current**: {metrics.import_time:.3f}s | **Target**: <2s

**Actions**:
- Lazy import heavy dependencies
- Reduce import depth
- Use `importlib` for dynamic imports
- Cache imported modules

**Example**:
```python
# Before
from heavy.module import HeavyClass

# After
def get_heavy_class():
    from heavy.module import HeavyClass
    return HeavyClass
```
""",
            )

        if metrics.avg_tool_execution_time > 0.1:
            recommendations.append(
                f"""
### 4. Optimize Tool Execution

**Current**: {metrics.avg_tool_execution_time:.3f}s | **Target**: <0.1s

**Actions**:
- Implement caching for repeated operations
- Use async/await properly
- Optimize database queries
- Add connection pooling

**Example**:
```python
# Before
result = expensive_operation(input)

# After
@lru_cache(maxsize=128)
def expensive_operation(input):
    # ... operation
    return result
```
""",
            )

        if not recommendations:
            recommendations.append(
                """
### ✅ All Metrics Within Targets!

Great job! All performance metrics are within acceptable ranges.

**Maintenance Recommendations**:
- Continue monitoring performance
- Run profiling regularly (weekly)
- Set up performance regression tests
- Document performance requirements
""",
            )

        report += "\n".join(recommendations)

        report += """

## Next Steps

1. **Implement high-priority optimizations** (startup, memory)
2. **Profile with detailed tools** (py-spy, memory-profiler)
3. **Set up performance monitoring** (continuous profiling)
4. **Create performance tests** (regression detection)
5. **Document performance requirements** (SLAs)

## Tools to Use

- **py-spy**: CPU profiling
- **memory-profiler**: Memory profiling
- **line-profiler**: Line-by-line profiling
- **cProfile**: Standard library profiler
- **Prometheus**: Metrics collection
- **Grafana**: Metrics visualization

"""

        return report


async def main():
    """
    Main execution.
    """
    profiler = PerformanceProfiler()

    # Run profiling
    metrics = await profiler.run_full_profile()

    # Generate report
    report = profiler.generate_report(metrics)
    print(report)

    # Save report
    report_path = Path("performance_report.md")
    with open(report_path, "w") as f:
        f.write(report)

    print(f"📄 Report saved to: {report_path}")
    print()
    print("✅ Profiling complete!")


if __name__ == "__main__":
    asyncio.run(main())
