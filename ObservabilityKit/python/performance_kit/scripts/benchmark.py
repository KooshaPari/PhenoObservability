#!/usr/bin/env python3
"""Performance Benchmark Script for Zen MCP Server.

This script measures performance of critical operations:
- Provider initialization and model selection
- Tool loading and execution overhead
- Conversation memory operations
- Authentication and encryption functions
- File I/O and storage operations
- Network request handling
"""

import asyncio
import hashlib
import json
import os
import statistics
import sys
import time
from pathlib import Path
from typing import Any

from src.shared.logging import get_logger

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    import aiohttp
    from auth.oauth2_encryption import OAuth2EncryptionManager
    from providers.registry import ModelProviderRegistry

    from src.shared.utilities.conversation_memory import ConversationMemoryManager
    from tools.shared.base_tool import BaseTool

logger = get_logger(__name__)


    IMPORTS_AVAILABLE = True
except ImportError as e:
    logger.info(f"Import error: {e}")
    logger.info("Some benchmarks may not be available")
    IMPORTS_AVAILABLE = False

# Configure logging
logging.basicConfig(level=logging.WARNING)  # Reduce noise during benchmarks


class PerformanceBenchmark:
    """
    Performance benchmark suite for Zen MCP Server.
    """

    def __init__(self):
        self.results: dict[str, dict[str, Any]] = {}

    def benchmark(self, name: str, func, *args, iterations: int = 100, **kwargs):
        """
        Run benchmark and record results.
        """
        logger.info(f"\n🔍 Benchmarking: {name}")
        times = []
        errors = 0

        # Warmup
        try:
            func(*args, **kwargs)
        except Exception:
            pass

        # Actual benchmark
        for _i in range(iterations):
            try:
                start_time = time.perf_counter()
                result = func(*args, **kwargs)
                if asyncio.iscoroutine(result):
                    result = asyncio.run(result)
                end_time = time.perf_counter()
                times.append((end_time - start_time) * 1000)  # Convert to milliseconds
            except Exception as e:
                errors += 1
                if errors > iterations * 0.1:  # Stop if >10% error rate
                    logger.info(f"❌ Too many errors, stopping benchmark: {e}")
                    break

        if not times:
            logger.info(f"❌ {name}: Failed to complete any iterations")
            return

        # Calculate statistics
        stats = {
            "mean_ms": statistics.mean(times),
            "median_ms": statistics.median(times),
            "std_dev_ms": statistics.stdev(times) if len(times) > 1 else 0,
            "min_ms": min(times),
            "max_ms": max(times),
            "p95_ms": times[int(len(times) * 0.95)] if len(times) > 20 else max(times),
            "iterations": len(times),
            "errors": errors,
            "ops_per_sec": 1000 / statistics.mean(times) if statistics.mean(times) > 0 else 0,
        }

        self.results[name] = stats
        print(f"✅ {name}: {stats['mean_ms']:.2f}ms avg, {stats['ops_per_sec']:.0f} ops/sec")

    def benchmark_crypto_operations(self):
        """
        Benchmark cryptographic operations.
        """
        logger.info("\n🔐 CRYPTOGRAPHIC OPERATIONS")

        # MD5 hashing (non-security)
        test_data = "test_data_for_hashing" * 100  # ~2KB
        self.benchmark(
            "MD5 Hashing (2KB)",
            lambda: hashlib.md5(test_data.encode(), usedforsecurity=False).hexdigest(),
            iterations=1000,
        )

        # SHA256 hashing
        self.benchmark("SHA256 Hashing (2KB)", lambda: hashlib.sha256(test_data.encode()).hexdigest(), iterations=1000)

        # OAuth2 encryption (if available)
        if IMPORTS_AVAILABLE:
            try:
                encryption_key = os.urandom(32)  # 256-bit key
                encryptor = OAuth2EncryptionManager(encryption_key=encryption_key)
                test_token = "test_oauth_token_data" * 10

                self.benchmark("OAuth2 Token Encryption", lambda: encryptor.encrypt_field(test_token), iterations=500)

                encrypted = encryptor.encrypt_field(test_token)
                self.benchmark("OAuth2 Token Decryption", lambda: encryptor.decrypt_field(encrypted), iterations=500)
            except Exception as e:
                logger.info(f"⚠️  OAuth2 encryption benchmark failed: {e}")

    def benchmark_storage_operations(self):
        """
        Benchmark storage and I/O operations.
        """
        logger.info("\n💾 STORAGE OPERATIONS")

        # File I/O operations
        test_data = {"test": "data", "numbers": list(range(100))}
        test_file = Path("/tmp/zen_benchmark_test.json")

        # Write operations
        self.benchmark("JSON File Write (small)", lambda: test_file.write_text(json.dumps(test_data)), iterations=200)

        # Read operations
        self.benchmark("JSON File Read (small)", lambda: json.loads(test_file.read_text()), iterations=200)

        # Cleanup
        try:
            test_file.unlink()
        except FileNotFoundError:
            pass

        # Large data operations
        large_data = {"data": "x" * 10000, "array": list(range(1000))}
        large_test_file = Path("/tmp/zen_benchmark_large.json")

        self.benchmark(
            "JSON File Write (large ~50KB)", lambda: large_test_file.write_text(json.dumps(large_data)), iterations=50
        )

        self.benchmark("JSON File Read (large ~50KB)", lambda: json.loads(large_test_file.read_text()), iterations=50)

        try:
            large_test_file.unlink()
        except FileNotFoundError:
            pass

    def benchmark_memory_operations(self):
        """
        Benchmark memory and conversation operations.
        """
        logger.info("\n🧠 MEMORY OPERATIONS")

        if not IMPORTS_AVAILABLE:
            logger.info("⚠️  Memory benchmarks unavailable - missing imports")
            return

        try:
            # Conversation memory operations
            memory = ConversationMemoryManager()

            # Memory storage
            test_message = {"role": "user", "content": "This is a test message for benchmarking purposes. " * 10}

            self.benchmark(
                "Store Conversation Message", lambda: memory.add_message("test_session", test_message), iterations=200
            )

            # Add some messages first
            for i in range(50):
                memory.add_message(
                    "benchmark_session", {"role": "user" if i % 2 == 0 else "assistant", "content": f"Message {i}"}
                )

            # Memory retrieval
            self.benchmark(
                "Retrieve Conversation History", lambda: memory.get_conversation("benchmark_session"), iterations=200
            )

        except Exception as e:
            logger.info(f"⚠️  Memory operations benchmark failed: {e}")

    def benchmark_provider_operations(self):
        """
        Benchmark AI provider operations.
        """
        logger.info("\n🤖 PROVIDER OPERATIONS")

        if not IMPORTS_AVAILABLE:
            logger.info("⚠️  Provider benchmarks unavailable - missing imports")
            return

        try:
            # Provider registry operations
            registry = ModelProviderRegistry()

            # Mock provider for testing
            class MockProvider:
                def __init__(self):
                    self.models = ["mock-model-1", "mock-model-2", "mock-model-3"]

                def get_available_models(self):
                    return self.models

                def supports_model(self, model: str) -> bool:
                    return model in self.models

            mock_provider = MockProvider()

            # Provider registration
            self.benchmark(
                "Provider Registration", lambda: registry.register_provider("mock", mock_provider), iterations=100
            )

            # Model lookup
            self.benchmark(
                "Model Availability Check", lambda: registry.get_provider_for_model("mock-model-1"), iterations=500
            )

            # Provider listing
            self.benchmark("List All Providers", lambda: registry.get_available_providers(), iterations=200)

        except Exception as e:
            logger.info(f"⚠️  Provider operations benchmark failed: {e}")

    def benchmark_tool_operations(self):
        """
        Benchmark tool loading and execution overhead.
        """
        logger.info("\n🔧 TOOL OPERATIONS")

        if not IMPORTS_AVAILABLE:
            logger.info("⚠️  Tool benchmarks unavailable - missing imports")
            return

        try:
            # Mock tool for testing
            class MockTool(BaseTool):
                def __init__(self):
                    super().__init__("mock_tool", "Mock tool for benchmarking")

                async def execute(self, arguments: dict[str, Any]) -> dict[str, Any]:
                    # Simulate some processing
                    await asyncio.sleep(0.001)  # 1ms simulated work
                    return {"result": "mock_result", "processed": len(str(arguments))}

            tool = MockTool()
            test_args = {"input": "test_input_data", "param": 42}

            # Tool execution overhead
            async def run_tool():
                return await tool.execute(test_args)

            self.benchmark("Tool Execution (mock)", lambda: asyncio.run(run_tool()), iterations=100)

            # Schema generation
            self.benchmark("Tool Schema Generation", lambda: tool.get_schema(), iterations=200)

        except Exception as e:
            logger.info(f"⚠️  Tool operations benchmark failed: {e}")

    async def benchmark_network_operations(self):
        """
        Benchmark network operations.
        """
        logger.info("\n🌐 NETWORK OPERATIONS")

        try:
            # HTTP client session setup
            self.benchmark("HTTP Session Creation", lambda: aiohttp.ClientSession(), iterations=50)

            # Local network request (to self)
            async def make_request():
                async with aiohttp.ClientSession() as session:
                    try:
                        async with session.get(
                            "http://httpbin.org/delay/0.1", timeout=aiohttp.ClientTimeout(total=5)
                        ) as response:
                            return await response.text()
                    except Exception:
                        return "timeout"

            # Note: This requires network access - may be skipped in CI
            logger.info("🌐 Testing network request (may timeout in restricted environments)")
            try:
                self.benchmark(
                    "HTTP GET Request (httpbin)",
                    lambda: asyncio.run(make_request()),
                    iterations=10,  # Lower iterations for network test
                )
            except Exception as e:
                logger.info(f"⚠️  Network benchmark skipped: {e}")

        except ImportError:
            logger.info("⚠️  Network benchmarks unavailable - aiohttp not installed")

    def print_summary(self):
        """
        Print benchmark summary.
        """
        print("\n" + "=" * 60)
        logger.info("📊 PERFORMANCE BENCHMARK SUMMARY")
        print("=" * 60)

        if not self.results:
            logger.info("❌ No benchmark results available")
            return

        # Sort by average time (fastest first)
        sorted_results = sorted(self.results.items(), key=lambda x: x[1]["mean_ms"])

        print(f"{'Operation':<30} {'Avg (ms)':<10} {'Ops/sec':<10} {'P95 (ms)':<10}")
        print("-" * 60)

        for name, stats in sorted_results:
            print(f"{name:<30} {stats['mean_ms']:<10.2f} {stats['ops_per_sec']:<10.0f} {stats['p95_ms']:<10.2f}")

        # Performance analysis
        logger.info("\n📈 PERFORMANCE ANALYSIS")
        print("-" * 30)

        # Find slowest operations
        slowest = sorted(self.results.items(), key=lambda x: x[1]["mean_ms"], reverse=True)[:3]

        logger.info("🐌 Slowest Operations:")
        for name, stats in slowest:
            if stats["mean_ms"] > 10:  # Over 10ms
                print(f"  • {name}: {stats['mean_ms']:.2f}ms")

        # Find fastest operations
        fastest = sorted_results[:3]
        logger.info("⚡ Fastest Operations:")
        for name, stats in fastest:
            if stats["mean_ms"] < 1:  # Under 1ms
                print(f"  • {name}: {stats['mean_ms']:.3f}ms")

        # Error summary
        total_errors = sum(stats["errors"] for stats in self.results.values())
        if total_errors > 0:
            logger.info(f"\n⚠️  Total Errors: {total_errors}")

        # Recommendations
        logger.info("\n💡 RECOMMENDATIONS")
        print("-" * 20)
        for name, stats in slowest:
            if stats["mean_ms"] > 50:
                print(f"  • Optimize '{name}' - {stats['mean_ms']:.1f}ms is slow")
            elif stats["std_dev_ms"] > stats["mean_ms"]:
                print(f"  • Investigate '{name}' variability - high std dev")

    def save_results(self, filepath: str | None = None):
        """
        Save benchmark results to JSON file.
        """
        if not filepath:
            timestamp = int(time.time())
            filepath = f"benchmark_results_{timestamp}.json"

        Path(filepath).write_text(json.dumps(self.results, indent=2))
        logger.info(f"💾 Results saved to: {filepath}")

    def run_all_benchmarks(self):
        """
        Run complete benchmark suite.
        """
        logger.info("🚀 Starting Zen MCP Server Performance Benchmarks")
        print("=" * 60)

        start_time = time.time()

        # Run all benchmark categories
        self.benchmark_crypto_operations()
        self.benchmark_storage_operations()
        self.benchmark_memory_operations()
        self.benchmark_provider_operations()
        self.benchmark_tool_operations()

        # Network benchmarks (may be skipped)
        try:
            asyncio.run(self.benchmark_network_operations())
        except Exception as e:
            logger.info(f"⚠️  Network benchmarks failed: {e}")

        total_time = time.time() - start_time
        logger.info(f"\n⏱️  Total benchmark time: {total_time:.1f} seconds")

        self.print_summary()
        self.save_results()


def main():
    """
    Main entry point.
    """
    benchmark = PerformanceBenchmark()

    try:
        benchmark.run_all_benchmarks()
        logger.info("\n✅ Benchmark completed successfully!")
        return 0
    except KeyboardInterrupt:
        logger.info("\n⚠️  Benchmark interrupted by user")
        if benchmark.results:
            benchmark.print_summary()
        return 1
    except Exception as e:
        logger.info(f"\n❌ Benchmark failed: {e}")
        return 1


if __name__ == "__main__":
    exit(main())
