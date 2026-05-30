#!/usr/bin/env python3
"""Dependency Analysis Script for ZEN-MCP-Server.

pipdeptree and pydeps integration for dependency visualization.
"""

import argparse
import json
import subprocess
from typing import Any


def run_dependency_analysis() -> dict[str, Any]:
    """
    Run comprehensive dependency analysis using pipdeptree and pydeps.
    """
    try:
        # Run pipdeptree for dependency tree
        deptree_result = subprocess.run(["pipdeptree", "--json"], check=False, capture_output=True, text=True)

        # Run pipdeptree for package info
        package_info_result = subprocess.run(
            ["pipdeptree", "--packages"], check=False, capture_output=True, text=True,
        )

        # Run pydeps for dependency graph (if available)
        pydeps_result = subprocess.run(
            ["pydeps", "src/", "--show-deps", "--format=json"], check=False, capture_output=True, text=True,
        )

        # Parse results
        deptree_data = []
        if deptree_result.returncode == 0 and deptree_result.stdout:
            try:
                deptree_data = json.loads(deptree_result.stdout)
            except json.JSONDecodeError:
                deptree_data = []

        package_info = package_info_result.stdout if package_info_result.returncode == 0 else ""

        pydeps_data = {}
        if pydeps_result.returncode == 0 and pydeps_result.stdout:
            try:
                pydeps_data = json.loads(pydeps_result.stdout)
            except json.JSONDecodeError:
                pydeps_data = {}

        # Analyze dependencies
        dependency_stats = analyze_dependency_patterns(deptree_data, package_info)

        return {
            "returncode": 0,
            "pipdeptree_data": deptree_data,
            "package_info": package_info,
            "pydeps_data": pydeps_data,
            "dependency_stats": dependency_stats,
            "recommendations": generate_dependency_recommendations(dependency_stats),
        }
    except Exception as e:
        return {"error": str(e), "returncode": 1}


def analyze_dependency_patterns(deptree_data: list[dict], package_info: str) -> dict[str, Any]:
    """
    Analyze dependency patterns and statistics.
    """
    total_packages = 0
    direct_dependencies = 0
    transitive_dependencies = 0
    dependency_conflicts = []
    outdated_packages = []
    security_issues = []

    # Analyze pipdeptree data
    if deptree_data:
        for package in deptree_data:
            total_packages += 1
            if package.get("package", {}).get("installed_version"):
                direct_dependencies += 1

            # Check for conflicts
            if "conflict" in package.get("package", {}):
                dependency_conflicts.append(package["package"])

            # Check for outdated packages
            if "latest_version" in package.get("package", {}):
                installed = package["package"].get("installed_version", "0.0.0")
                latest = package["package"].get("latest_version", "0.0.0")
                if installed != latest:
                    outdated_packages.append(
                        {
                            "name": package["package"].get("package_name", ""),
                            "installed": installed,
                            "latest": latest,
                        },
                    )

    # Count transitive dependencies
    transitive_dependencies = total_packages - direct_dependencies

    # Analyze package info for security issues
    if package_info:
        lines = package_info.split("\n")
        for line in lines:
            if "security" in line.lower() or "vulnerability" in line.lower():
                security_issues.append(line.strip())

    return {
        "total_packages": total_packages,
        "direct_dependencies": direct_dependencies,
        "transitive_dependencies": transitive_dependencies,
        "dependency_conflicts": dependency_conflicts,
        "outdated_packages": outdated_packages,
        "security_issues": security_issues,
        "dependency_depth": calculate_dependency_depth(deptree_data),
        "circular_dependencies": detect_circular_dependencies(deptree_data),
    }


def calculate_dependency_depth(deptree_data: list[dict]) -> dict[str, Any]:
    """
    Calculate dependency depth statistics.
    """
    if not deptree_data:
        return {"max_depth": 0, "average_depth": 0}

    depths = []
    for package in deptree_data:
        depth = calculate_package_depth(package, 0)
        depths.append(depth)

    return {
        "max_depth": max(depths) if depths else 0,
        "average_depth": sum(depths) / len(depths) if depths else 0,
        "depth_distribution": {
            "shallow": len([d for d in depths if d <= 2]),
            "medium": len([d for d in depths if 2 < d <= 5]),
            "deep": len([d for d in depths if d > 5]),
        },
    }


def calculate_package_depth(package: dict, current_depth: int) -> int:
    """
    Calculate depth of a package in the dependency tree.
    """
    max_depth = current_depth

    for dependency in package.get("dependencies", []):
        depth = calculate_package_depth(dependency, current_depth + 1)
        max_depth = max(max_depth, depth)

    return max_depth


def detect_circular_dependencies(deptree_data: list[dict]) -> list[dict[str, Any]]:
    """
    Detect circular dependencies.
    """
    circular_deps = []

    # Simple circular dependency detection
    # In a real implementation, this would use graph algorithms
    for package in deptree_data:
        package_name = package.get("package", {}).get("package_name", "")
        if package_name:
            # Check if this package appears in its own dependency chain
            if check_circular_dependency(package, package_name, set()):
                circular_deps.append(
                    {"package": package_name, "cycle": [package_name]},  # Simplified
                )

    return circular_deps


def check_circular_dependency(package: dict, target_name: str, visited: set) -> bool:
    """
    Check if a package has a circular dependency.
    """
    package_name = package.get("package", {}).get("package_name", "")

    if package_name in visited:
        return package_name == target_name

    visited.add(package_name)

    for dependency in package.get("dependencies", []):
        if check_circular_dependency(dependency, target_name, visited.copy()):
            return True

    return False


def generate_dependency_recommendations(stats: dict) -> list[str]:
    """
    Generate dependency management recommendations.
    """
    recommendations = []

    total_packages = stats.get("total_packages", 0)
    outdated_packages = stats.get("outdated_packages", [])
    dependency_conflicts = stats.get("dependency_conflicts", [])
    security_issues = stats.get("security_issues", [])
    circular_deps = stats.get("circular_dependencies", [])

    if total_packages > 100:
        recommendations.append(
            f"High number of dependencies ({total_packages}). Consider consolidating or removing unused packages",
        )

    if outdated_packages:
        recommendations.append(
            f"Update {len(outdated_packages)} outdated packages to latest versions",
        )
        for pkg in outdated_packages[:3]:  # Show top 3
            recommendations.append(f"  - {pkg['name']}: {pkg['installed']} → {pkg['latest']}")

    if dependency_conflicts:
        recommendations.append(f"Resolve {len(dependency_conflicts)} dependency conflicts")

    if security_issues:
        recommendations.append(f"Address {len(security_issues)} security issues in dependencies")

    if circular_deps:
        recommendations.append(f"Resolve {len(circular_deps)} circular dependencies")

    # General recommendations
    recommendations.extend(
        [
            "Regularly audit dependencies for security vulnerabilities",
            "Use dependency pinning for production deployments",
            "Consider using virtual environments for isolation",
            "Monitor dependency updates for breaking changes",
            "Use tools like safety and pip-audit for security scanning",
        ],
    )

    return recommendations


def generate_dependency_report() -> str:
    """
    Generate a comprehensive dependency analysis report.
    """
    analysis = run_dependency_analysis()

    report = []
    report.append("ZEN-MCP-Server Dependency Analysis Report")
    report.append("=" * 50)

    if "error" in analysis:
        report.append(f"Error: {analysis['error']}")
        return "\n".join(report)

    stats = analysis.get("dependency_stats", {})
    recommendations = analysis.get("recommendations", [])

    report.append(f"Total Packages: {stats.get('total_packages', 0)}")
    report.append(f"Direct Dependencies: {stats.get('direct_dependencies', 0)}")
    report.append(f"Transitive Dependencies: {stats.get('transitive_dependencies', 0)}")

    depth_info = stats.get("dependency_depth", {})
    report.append(f"Max Dependency Depth: {depth_info.get('max_depth', 0)}")
    report.append(f"Average Dependency Depth: {depth_info.get('average_depth', 0):.2f}")

    outdated_packages = stats.get("outdated_packages", [])
    if outdated_packages:
        report.append(f"\nOutdated Packages: {len(outdated_packages)}")
        for pkg in outdated_packages[:5]:  # Show top 5
            report.append(f"  {pkg['name']}: {pkg['installed']} → {pkg['latest']}")

    dependency_conflicts = stats.get("dependency_conflicts", [])
    if dependency_conflicts:
        report.append(f"\nDependency Conflicts: {len(dependency_conflicts)}")
        for conflict in dependency_conflicts[:3]:  # Show top 3
            report.append(
                f"  {conflict.get('package_name', 'Unknown')}: {conflict.get('conflict', 'Unknown')}",
            )

    security_issues = stats.get("security_issues", [])
    if security_issues:
        report.append(f"\nSecurity Issues: {len(security_issues)}")
        for issue in security_issues[:3]:  # Show top 3
            report.append(f"  {issue}")

    circular_deps = stats.get("circular_dependencies", [])
    if circular_deps:
        report.append(f"\nCircular Dependencies: {len(circular_deps)}")
        for dep in circular_deps:
            report.append(f"  {dep['package']}: {dep['cycle']}")

    if recommendations:
        report.append("\nRecommendations:")
        for i, rec in enumerate(recommendations, 1):
            report.append(f"  {i}. {rec}")

    return "\n".join(report)


def main():
    """
    Main dependency analysis function.
    """
    parser = argparse.ArgumentParser(description="Analyze dependencies")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")

    args = parser.parse_args()

    if args.report:
        report = generate_dependency_report()
        print(report)
        return 0

    analysis = run_dependency_analysis()

    if args.json:
        print(json.dumps(analysis, indent=2))
    else:
        print("Dependency Analysis Results:")
        stats = analysis.get("dependency_stats", {})
        print(f"  Total Packages: {stats.get('total_packages', 0)}")
        print(f"  Direct Dependencies: {stats.get('direct_dependencies', 0)}")
        print(f"  Outdated Packages: {len(stats.get('outdated_packages', []))}")
        print(f"  Security Issues: {len(stats.get('security_issues', []))}")

    return analysis.get("returncode", 1)


if __name__ == "__main__":
    raise SystemExit(main())
