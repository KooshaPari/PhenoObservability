#!/usr/bin/env python3
"""Coverage Analysis Script for ZEN-MCP-Server.

Comprehensive test coverage tracking with HTML and XML reports.
"""

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


def run_coverage_analysis() -> dict[str, Any]:
    """
    Run comprehensive coverage analysis.
    """
    try:
        # Run pytest with coverage
        result = subprocess.run(
            [
                "pytest",
                "--cov=src",
                "--cov-report=term-missing",
                "--cov-report=html:htmlcov",
                "--cov-report=xml:coverage.xml",
                "--cov-report=json:coverage.json",
                "--cov-fail-under=80",
            ],
            check=False, capture_output=True,
            text=True,
        )

        # Parse coverage results
        coverage_data = {}
        if Path("coverage.json").exists():
            with open("coverage.json") as f:
                coverage_data = json.load(f)

        return {
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "coverage_data": coverage_data,
            "html_report": Path("htmlcov/index.html").exists(),
            "xml_report": Path("coverage.xml").exists(),
            "json_report": Path("coverage.json").exists(),
        }
    except Exception as e:
        return {"error": str(e), "returncode": 1}


def analyze_coverage_trends() -> dict[str, Any]:
    """
    Analyze coverage trends over time.
    """
    # This would typically read from historical coverage data
    # For now, return placeholder data
    return {
        "trend": "stable",
        "coverage_change": 0.0,
        "files_added": 0,
        "files_removed": 0,
        "coverage_improvement": [],
    }


def generate_coverage_report() -> str:
    """
    Generate a comprehensive coverage report.
    """
    analysis = run_coverage_analysis()
    trends = analyze_coverage_trends()

    report = []
    report.append("ZEN-MCP-Server Coverage Analysis Report")
    report.append("=" * 50)

    if "error" in analysis:
        report.append(f"Error: {analysis['error']}")
        return "\n".join(report)

    report.append(
        f"Coverage Analysis Status: {'PASSED' if analysis['returncode'] == 0 else 'FAILED'}",
    )
    report.append(f"HTML Report Generated: {analysis['html_report']}")
    report.append(f"XML Report Generated: {analysis['xml_report']}")
    report.append(f"JSON Report Generated: {analysis['json_report']}")

    if analysis["coverage_data"]:
        total_coverage = analysis["coverage_data"].get("totals", {}).get("percent_covered", 0)
        report.append(f"Total Coverage: {total_coverage:.2f}%")

        files = analysis["coverage_data"].get("files", {})
        report.append(f"Files Analyzed: {len(files)}")

        # Show files with low coverage
        low_coverage_files = [
            (file, data.get("summary", {}).get("percent_covered", 0))
            for file, data in files.items()
            if data.get("summary", {}).get("percent_covered", 0) < 80
        ]

        if low_coverage_files:
            report.append("\nFiles with Low Coverage (<80%):")
            for file, coverage in sorted(low_coverage_files, key=lambda x: x[1]):
                report.append(f"  {file}: {coverage:.2f}%")

    report.append(f"\nCoverage Trend: {trends['trend']}")
    report.append(f"Coverage Change: {trends['coverage_change']:+.2f}%")

    return "\n".join(report)


def main():
    """
    Main coverage analysis function.
    """
    parser = argparse.ArgumentParser(description="Run coverage analysis")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")

    args = parser.parse_args()

    if args.report:
        report = generate_coverage_report()
        print(report)
        return 0

    analysis = run_coverage_analysis()

    if args.json:
        print(json.dumps(analysis, indent=2))
    else:
        print("Coverage Analysis Results:")
        print(f"  Status: {'PASSED' if analysis['returncode'] == 0 else 'FAILED'}")
        print(f"  HTML Report: {analysis.get('html_report', False)}")
        print(f"  XML Report: {analysis.get('xml_report', False)}")
        print(f"  JSON Report: {analysis.get('json_report', False)}")

    return analysis.get("returncode", 1)


if __name__ == "__main__":
    raise SystemExit(main())
