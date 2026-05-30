#!/usr/bin/env python3
"""Test Duration Tracking Script for ZEN-MCP-Server.

Automated slow test detection and duration reporting.
"""

import argparse
import json
import subprocess
from typing import Any


def run_test_duration_analysis() -> dict[str, Any]:
    """
    Run test duration analysis.
    """
    try:
        # Run pytest with duration tracking
        result = subprocess.run(
            ["pytest", "--durations=0", "--durations-min=0.1", "-v"], check=False, capture_output=True, text=True,
        )

        # Parse duration information from output
        durations = []
        slow_tests = []

        for line in result.stdout.split("\n"):
            if "passed" in line and "[" in line and "]" in line:
                # Extract test name and duration
                parts = line.split("[")
                if len(parts) >= 2:
                    test_name = parts[0].strip()
                    duration_part = parts[1].split("]")[0]
                    try:
                        duration = float(duration_part.replace("s", ""))
                        durations.append({"test": test_name, "duration": duration})
                        if duration > 5.0:  # Consider tests > 5s as slow
                            slow_tests.append({"test": test_name, "duration": duration})
                    except ValueError:
                        continue

        # Sort by duration (slowest first)
        durations.sort(key=lambda x: x["duration"], reverse=True)
        slow_tests.sort(key=lambda x: x["duration"], reverse=True)

        return {
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "total_tests": len(durations),
            "slow_tests": slow_tests,
            "slow_test_count": len(slow_tests),
            "average_duration": (
                sum(d["duration"] for d in durations) / len(durations) if durations else 0
            ),
            "max_duration": max(d["duration"] for d in durations) if durations else 0,
            "all_durations": durations,
        }
    except Exception as e:
        return {"error": str(e), "returncode": 1}


def identify_slow_tests(threshold: float = 5.0) -> list[dict[str, Any]]:
    """
    Identify tests that exceed the duration threshold.
    """
    analysis = run_test_duration_analysis()
    if "error" in analysis:
        return []

    return [test for test in analysis.get("all_durations", []) if test["duration"] > threshold]


def generate_duration_report() -> str:
    """
    Generate a comprehensive duration report.
    """
    analysis = run_test_duration_analysis()

    report = []
    report.append("ZEN-MCP-Server Test Duration Report")
    report.append("=" * 40)

    if "error" in analysis:
        report.append(f"Error: {analysis['error']}")
        return "\n".join(report)

    report.append(f"Total Tests: {analysis['total_tests']}")
    report.append(f"Slow Tests (>5s): {analysis['slow_test_count']}")
    report.append(f"Average Duration: {analysis['average_duration']:.2f}s")
    report.append(f"Max Duration: {analysis['max_duration']:.2f}s")

    if analysis["slow_tests"]:
        report.append("\nSlowest Tests:")
        for i, test in enumerate(analysis["slow_tests"][:10], 1):  # Show top 10
            report.append(f"  {i:2d}. {test['test']}: {test['duration']:.2f}s")

    # Performance recommendations
    if analysis["slow_test_count"] > 0:
        report.append("\nRecommendations:")
        report.append("  - Consider optimizing slow tests")
        report.append("  - Use @pytest.mark.slow for long-running tests")
        report.append("  - Consider parallel execution for independent tests")

    return "\n".join(report)


def main():
    """
    Main duration tracking function.
    """
    parser = argparse.ArgumentParser(description="Track test durations")
    parser.add_argument(
        "--threshold", type=float, default=5.0, help="Slow test threshold in seconds",
    )
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")

    args = parser.parse_args()

    if args.report:
        report = generate_duration_report()
        print(report)
        return 0

    analysis = run_test_duration_analysis()

    if args.json:
        print(json.dumps(analysis, indent=2))
    else:
        print("Test Duration Analysis Results:")
        print(f"  Total Tests: {analysis.get('total_tests', 0)}")
        print(f"  Slow Tests: {analysis.get('slow_test_count', 0)}")
        print(f"  Average Duration: {analysis.get('average_duration', 0):.2f}s")
        print(f"  Max Duration: {analysis.get('max_duration', 0):.2f}s")

    return analysis.get("returncode", 1)


if __name__ == "__main__":
    raise SystemExit(main())
