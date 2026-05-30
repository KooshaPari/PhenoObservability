#!/usr/bin/env python3
"""Code Complexity Analysis Script for ZEN-MCP-Server.

Radon integration for automated complexity analysis.
"""

import argparse
import json
import subprocess
from typing import Any


def run_complexity_analysis() -> dict[str, Any]:
    """
    Run comprehensive complexity analysis using Radon.
    """
    try:
        # Run radon cc (cyclomatic complexity)
        cc_result = subprocess.run(
            ["radon", "cc", "src/", "-a", "-j"], check=False, capture_output=True, text=True,
        )

        # Run radon mi (maintainability index)
        mi_result = subprocess.run(["radon", "mi", "src/", "-j"], check=False, capture_output=True, text=True)

        # Run radon hal (Halstead complexity)
        hal_result = subprocess.run(["radon", "hal", "src/", "-j"], check=False, capture_output=True, text=True)

        # Parse results
        cc_data = json.loads(cc_result.stdout) if cc_result.returncode == 0 else {}
        mi_data = json.loads(mi_result.stdout) if mi_result.returncode == 0 else {}
        hal_data = json.loads(hal_result.stdout) if hal_result.returncode == 0 else {}

        # Calculate complexity statistics
        complexity_stats = calculate_complexity_stats(cc_data)
        maintainability_stats = calculate_maintainability_stats(mi_data)
        halstead_stats = calculate_halstead_stats(hal_data)

        return {
            "returncode": 0,
            "cyclomatic_complexity": {"data": cc_data, "stats": complexity_stats},
            "maintainability_index": {"data": mi_data, "stats": maintainability_stats},
            "halstead_complexity": {"data": hal_data, "stats": halstead_stats},
            "overall_complexity_score": calculate_overall_score(
                complexity_stats, maintainability_stats, halstead_stats,
            ),
        }
    except Exception as e:
        return {"error": str(e), "returncode": 1}


def calculate_complexity_stats(cc_data: dict) -> dict[str, Any]:
    """
    Calculate cyclomatic complexity statistics.
    """
    if not cc_data:
        return {}

    all_complexities = []
    high_complexity_files = []

    for file_path, functions in cc_data.items():
        for func_name, func_data in functions.items():
            complexity = func_data.get("complexity", 0)
            all_complexities.append(complexity)

            if complexity > 10:  # High complexity threshold
                high_complexity_files.append(
                    {"file": file_path, "function": func_name, "complexity": complexity},
                )

    if not all_complexities:
        return {}

    return {
        "total_functions": len(all_complexities),
        "average_complexity": sum(all_complexities) / len(all_complexities),
        "max_complexity": max(all_complexities),
        "min_complexity": min(all_complexities),
        "high_complexity_count": len(high_complexity_files),
        "high_complexity_functions": high_complexity_files[:10],  # Top 10
    }


def calculate_maintainability_stats(mi_data: dict) -> dict[str, Any]:
    """
    Calculate maintainability index statistics.
    """
    if not mi_data:
        return {}

    all_indices = []
    low_maintainability_files = []

    for file_path, file_data in mi_data.items():
        if isinstance(file_data, dict) and "mi" in file_data:
            mi_value = file_data["mi"]
            all_indices.append(mi_value)

            if mi_value < 20:  # Low maintainability threshold
                low_maintainability_files.append(
                    {"file": file_path, "maintainability_index": mi_value},
                )

    if not all_indices:
        return {}

    return {
        "total_files": len(all_indices),
        "average_mi": sum(all_indices) / len(all_indices),
        "max_mi": max(all_indices),
        "min_mi": min(all_indices),
        "low_maintainability_count": len(low_maintainability_files),
        "low_maintainability_files": low_maintainability_files[:10],  # Top 10
    }


def calculate_halstead_stats(hal_data: dict) -> dict[str, Any]:
    """
    Calculate Halstead complexity statistics.
    """
    if not hal_data:
        return {}

    all_efforts = []
    high_effort_files = []

    for file_path, file_data in hal_data.items():
        if isinstance(file_data, dict) and "effort" in file_data:
            effort = file_data["effort"]
            all_efforts.append(effort)

            if effort > 1000:  # High effort threshold
                high_effort_files.append({"file": file_path, "effort": effort})

    if not all_efforts:
        return {}

    return {
        "total_files": len(all_efforts),
        "average_effort": sum(all_efforts) / len(all_efforts),
        "max_effort": max(all_efforts),
        "min_effort": min(all_efforts),
        "high_effort_count": len(high_effort_files),
        "high_effort_files": high_effort_files[:10],  # Top 10
    }


def calculate_overall_score(cc_stats: dict, mi_stats: dict, hal_stats: dict) -> dict[str, Any]:
    """
    Calculate overall complexity score.
    """
    score = 100  # Start with perfect score

    # Deduct points for high complexity
    if cc_stats.get("high_complexity_count", 0) > 0:
        score -= min(cc_stats["high_complexity_count"] * 2, 30)

    # Deduct points for low maintainability
    if mi_stats.get("low_maintainability_count", 0) > 0:
        score -= min(mi_stats["low_maintainability_count"] * 3, 40)

    # Deduct points for high effort
    if hal_stats.get("high_effort_count", 0) > 0:
        score -= min(hal_stats["high_effort_count"] * 1, 20)

    # Determine grade
    if score >= 90:
        grade = "A"
    elif score >= 80:
        grade = "B"
    elif score >= 70:
        grade = "C"
    elif score >= 60:
        grade = "D"
    else:
        grade = "F"

    return {
        "score": max(score, 0),
        "grade": grade,
        "recommendations": generate_recommendations(cc_stats, mi_stats, hal_stats),
    }


def generate_recommendations(cc_stats: dict, mi_stats: dict, hal_stats: dict) -> list[str]:
    """
    Generate complexity improvement recommendations.
    """
    recommendations = []

    if cc_stats.get("high_complexity_count", 0) > 0:
        recommendations.append(
            f"Refactor {cc_stats['high_complexity_count']} high-complexity functions",
        )

    if mi_stats.get("low_maintainability_count", 0) > 0:
        recommendations.append(
            f"Improve maintainability of {mi_stats['low_maintainability_count']} files",
        )

    if hal_stats.get("high_effort_count", 0) > 0:
        recommendations.append(f"Simplify {hal_stats['high_effort_count']} high-effort files")

    if not recommendations:
        recommendations.append("Code complexity is within acceptable limits")

    return recommendations


def generate_complexity_report() -> str:
    """
    Generate a comprehensive complexity report.
    """
    analysis = run_complexity_analysis()

    report = []
    report.append("ZEN-MCP-Server Code Complexity Analysis Report")
    report.append("=" * 50)

    if "error" in analysis:
        report.append(f"Error: {analysis['error']}")
        return "\n".join(report)

    # Overall score
    overall = analysis.get("overall_complexity_score", {})
    report.append(
        f"Overall Complexity Score: {overall.get('score', 0)}/100 ({overall.get('grade', 'N/A')})",
    )

    # Cyclomatic complexity
    cc_stats = analysis.get("cyclomatic_complexity", {}).get("stats", {})
    if cc_stats:
        report.append("\nCyclomatic Complexity:")
        report.append(f"  Total Functions: {cc_stats.get('total_functions', 0)}")
        report.append(f"  Average Complexity: {cc_stats.get('average_complexity', 0):.2f}")
        report.append(f"  Max Complexity: {cc_stats.get('max_complexity', 0)}")
        report.append(f"  High Complexity Functions: {cc_stats.get('high_complexity_count', 0)}")

    # Maintainability index
    mi_stats = analysis.get("maintainability_index", {}).get("stats", {})
    if mi_stats:
        report.append("\nMaintainability Index:")
        report.append(f"  Total Files: {mi_stats.get('total_files', 0)}")
        report.append(f"  Average MI: {mi_stats.get('average_mi', 0):.2f}")
        report.append(f"  Max MI: {mi_stats.get('max_mi', 0):.2f}")
        report.append(
            f"  Low Maintainability Files: {mi_stats.get('low_maintainability_count', 0)}",
        )

    # Halstead complexity
    hal_stats = analysis.get("halstead_complexity", {}).get("stats", {})
    if hal_stats:
        report.append("\nHalstead Complexity:")
        report.append(f"  Total Files: {hal_stats.get('total_files', 0)}")
        report.append(f"  Average Effort: {hal_stats.get('average_effort', 0):.2f}")
        report.append(f"  Max Effort: {hal_stats.get('max_effort', 0):.2f}")
        report.append(f"  High Effort Files: {hal_stats.get('high_effort_count', 0)}")

    # Recommendations
    recommendations = overall.get("recommendations", [])
    if recommendations:
        report.append("\nRecommendations:")
        for i, rec in enumerate(recommendations, 1):
            report.append(f"  {i}. {rec}")

    return "\n".join(report)


def main():
    """
    Main complexity analysis function.
    """
    parser = argparse.ArgumentParser(description="Analyze code complexity")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")

    args = parser.parse_args()

    if args.report:
        report = generate_complexity_report()
        print(report)
        return 0

    analysis = run_complexity_analysis()

    if args.json:
        print(json.dumps(analysis, indent=2))
    else:
        print("Code Complexity Analysis Results:")
        overall = analysis.get("overall_complexity_score", {})
        print(f"  Overall Score: {overall.get('score', 0)}/100 ({overall.get('grade', 'N/A')})")

        cc_stats = analysis.get("cyclomatic_complexity", {}).get("stats", {})
        if cc_stats:
            print(f"  High Complexity Functions: {cc_stats.get('high_complexity_count', 0)}")

        mi_stats = analysis.get("maintainability_index", {}).get("stats", {})
        if mi_stats:
            print(f"  Low Maintainability Files: {mi_stats.get('low_maintainability_count', 0)}")

    return analysis.get("returncode", 1)


if __name__ == "__main__":
    raise SystemExit(main())
