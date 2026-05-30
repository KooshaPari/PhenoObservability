#!/usr/bin/env python3
"""Code Duplication Analysis Script for ZEN-MCP-Server.

Pylint duplicate code detection and refactoring recommendations.
"""

import argparse
import json
import subprocess
from typing import Any


def run_duplication_analysis() -> dict[str, Any]:
    """
    Run comprehensive duplication analysis using Pylint.
    """
    try:
        # Run pylint with duplicate code detection
        result = subprocess.run(
            ["pylint", "src/", "--disable=all", "--enable=duplicate-code", "--output-format=json"],
            check=False, capture_output=True,
            text=True,
        )

        # Parse pylint output
        pylint_data = []
        if result.stdout:
            for line in result.stdout.strip().split("\n"):
                if line.strip():
                    try:
                        pylint_data.append(json.loads(line))
                    except json.JSONDecodeError:
                        continue

        # Analyze duplication patterns
        duplication_stats = analyze_duplication_patterns(pylint_data)

        return {
            "returncode": result.returncode,
            "pylint_output": pylint_data,
            "duplication_stats": duplication_stats,
            "recommendations": generate_refactoring_recommendations(duplication_stats),
        }
    except Exception as e:
        return {"error": str(e), "returncode": 1}


def analyze_duplication_patterns(pylint_data: list[dict]) -> dict[str, Any]:
    """
    Analyze duplication patterns from pylint output.
    """
    duplicate_blocks = []
    duplicate_functions = []
    duplicate_classes = []

    for item in pylint_data:
        if item.get("type") == "duplicate-code":
            message = item.get("message", "")
            if "duplicate code" in message.lower():
                duplicate_blocks.append(
                    {
                        "file": item.get("path", ""),
                        "line": item.get("line", 0),
                        "message": message,
                        "confidence": item.get("confidence", 0),
                    },
                )

    # Group duplicates by similarity
    similar_blocks = group_similar_duplicates(duplicate_blocks)

    return {
        "total_duplicate_blocks": len(duplicate_blocks),
        "duplicate_blocks": duplicate_blocks,
        "similar_groups": similar_blocks,
        "duplication_percentage": calculate_duplication_percentage(duplicate_blocks),
    }


def group_similar_duplicates(duplicate_blocks: list[dict]) -> list[dict[str, Any]]:
    """
    Group similar duplicate blocks together.
    """
    groups = []
    processed = set()

    for i, block1 in enumerate(duplicate_blocks):
        if i in processed:
            continue

        group = [block1]
        processed.add(i)

        for j, block2 in enumerate(duplicate_blocks[i + 1 :], i + 1):
            if j in processed:
                continue

            # Simple similarity check based on file path and message
            if block1["file"] == block2["file"] or similar_message(
                block1["message"], block2["message"],
            ):
                group.append(block2)
                processed.add(j)

        if len(group) > 1:
            groups.append(
                {
                    "group_id": len(groups) + 1,
                    "blocks": group,
                    "similarity_score": calculate_group_similarity(group),
                },
            )

    return groups


def similar_message(msg1: str, msg2: str) -> bool:
    """
    Check if two messages are similar.
    """
    # Simple similarity check - in real implementation, use more sophisticated algorithms
    words1 = set(msg1.lower().split())
    words2 = set(msg2.lower().split())
    intersection = words1.intersection(words2)
    union = words1.union(words2)

    if len(union) == 0:
        return False

    similarity = len(intersection) / len(union)
    return similarity > 0.5


def calculate_group_similarity(group: list[dict]) -> float:
    """
    Calculate similarity score for a group of duplicate blocks.
    """
    if len(group) < 2:
        return 0.0

    total_similarity = 0.0
    comparisons = 0

    for i in range(len(group)):
        for j in range(i + 1, len(group)):
            msg1 = group[i]["message"]
            msg2 = group[j]["message"]
            words1 = set(msg1.lower().split())
            words2 = set(msg2.lower().split())
            intersection = words1.intersection(words2)
            union = words1.union(words2)

            if len(union) > 0:
                similarity = len(intersection) / len(union)
                total_similarity += similarity
                comparisons += 1

    return total_similarity / comparisons if comparisons > 0 else 0.0


def calculate_duplication_percentage(duplicate_blocks: list[dict]) -> float:
    """
    Calculate overall duplication percentage.
    """
    # This is a simplified calculation
    # In real implementation, would analyze actual code lines
    if not duplicate_blocks:
        return 0.0

    # Estimate based on number of duplicate blocks
    # Assume each duplicate block represents ~10 lines of duplicated code
    estimated_duplicated_lines = len(duplicate_blocks) * 10
    estimated_total_lines = 1000  # This would be calculated from actual code

    return min((estimated_duplicated_lines / estimated_total_lines) * 100, 100.0)


def generate_refactoring_recommendations(duplication_stats: dict) -> list[str]:
    """
    Generate refactoring recommendations based on duplication analysis.
    """
    recommendations = []

    total_blocks = duplication_stats.get("total_duplicate_blocks", 0)
    similar_groups = duplication_stats.get("similar_groups", [])
    duplication_percentage = duplication_stats.get("duplication_percentage", 0)

    if total_blocks == 0:
        recommendations.append("No significant code duplication detected")
        return recommendations

    if duplication_percentage > 10:
        recommendations.append(
            f"High duplication detected ({duplication_percentage:.1f}%). Consider extracting common functionality into shared modules",
        )

    if len(similar_groups) > 0:
        recommendations.append(
            f"Found {len(similar_groups)} groups of similar duplicate code. Consider creating utility functions or base classes",
        )

    # Specific recommendations based on duplicate patterns
    for group in similar_groups:
        blocks = group["blocks"]
        files = set(block["file"] for block in blocks)

        if len(files) > 1:
            recommendations.append(
                f"Duplicate code found across {len(files)} files. Consider creating a shared utility module",
            )
        else:
            recommendations.append(
                f"Duplicate code found within {list(files)[0]}. Consider extracting into separate functions",
            )

    # General recommendations
    recommendations.extend(
        [
            "Use DRY (Don't Repeat Yourself) principle to reduce duplication",
            "Consider using inheritance or composition to share common functionality",
            "Extract common patterns into utility functions or classes",
            "Use configuration files for repeated constants or settings",
        ],
    )

    return recommendations


def generate_duplication_report() -> str:
    """
    Generate a comprehensive duplication analysis report.
    """
    analysis = run_duplication_analysis()

    report = []
    report.append("ZEN-MCP-Server Code Duplication Analysis Report")
    report.append("=" * 50)

    if "error" in analysis:
        report.append(f"Error: {analysis['error']}")
        return "\n".join(report)

    stats = analysis.get("duplication_stats", {})
    recommendations = analysis.get("recommendations", [])

    report.append(f"Total Duplicate Blocks: {stats.get('total_duplicate_blocks', 0)}")
    report.append(f"Duplication Percentage: {stats.get('duplication_percentage', 0):.2f}%")

    similar_groups = stats.get("similar_groups", [])
    if similar_groups:
        report.append(f"\nSimilar Code Groups: {len(similar_groups)}")
        for i, group in enumerate(similar_groups[:5], 1):  # Show top 5
            blocks = group["blocks"]
            files = set(block["file"] for block in blocks)
            report.append(
                f"  Group {i}: {len(blocks)} blocks across {len(files)} files (similarity: {group['similarity_score']:.2f})",
            )

    duplicate_blocks = stats.get("duplicate_blocks", [])
    if duplicate_blocks:
        report.append("\nDuplicate Code Locations:")
        for block in duplicate_blocks[:10]:  # Show top 10
            report.append(f"  {block['file']}:{block['line']} - {block['message'][:100]}...")

    if recommendations:
        report.append("\nRefactoring Recommendations:")
        for i, rec in enumerate(recommendations, 1):
            report.append(f"  {i}. {rec}")

    return "\n".join(report)


def main():
    """
    Main duplication analysis function.
    """
    parser = argparse.ArgumentParser(description="Analyze code duplication")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")

    args = parser.parse_args()

    if args.report:
        report = generate_duplication_report()
        print(report)
        return 0

    analysis = run_duplication_analysis()

    if args.json:
        print(json.dumps(analysis, indent=2))
    else:
        print("Code Duplication Analysis Results:")
        stats = analysis.get("duplication_stats", {})
        print(f"  Duplicate Blocks: {stats.get('total_duplicate_blocks', 0)}")
        print(f"  Duplication %: {stats.get('duplication_percentage', 0):.2f}%")

        similar_groups = stats.get("similar_groups", [])
        print(f"  Similar Groups: {len(similar_groups)}")

    return analysis.get("returncode", 1)


if __name__ == "__main__":
    raise SystemExit(main())
