#!/usr/bin/env python3
"""
Analyze current test structure and plan consolidation.
"""

from pathlib import Path


def analyze_tests():
    test_dir = Path("tests")

    # Categorize test files
    categories = {
        "unit": [],
        "integration": [],
        "e2e": [],
        "performance": [],
        "simulator": [],
        "user_stories": [],
        "uncategorized": [],
    }

    # Patterns for categorization
    integration_patterns = ["integration", "e2e", "end_to_end", "full_system"]
    performance_patterns = ["performance", "load", "stress", "benchmark"]
    unit_patterns = ["unit/", "mock", "isolated"]

    for test_file in test_dir.rglob("test_*.py"):
        file_path = str(test_file)

        if "unit/" in file_path:
            categories["unit"].append(test_file)
        elif "integration" in file_path:
            categories["integration"].append(test_file)
        elif "simulator" in file_path:
            categories["simulator"].append(test_file)
        elif "user_stories" in file_path:
            categories["user_stories"].append(test_file)
        elif any(p in file_path.lower() for p in performance_patterns):
            categories["performance"].append(test_file)
        elif any(p in file_path.lower() for p in integration_patterns):
            categories["e2e"].append(test_file)
        # Check file location - root level tests
        elif test_file.parent == test_dir:
            categories["uncategorized"].append(test_file)

    print("📊 Test File Analysis")
    print("=" * 60)
    for category, files in categories.items():
        print(f"\n{category.upper()}: {len(files)} files")
        if len(files) <= 10:
            for f in files:
                print(f"  - {f.relative_to(test_dir)}")

    print(f"\n{'=' * 60}")
    print(f"TOTAL: {sum(len(files) for files in categories.values())} test files analyzed")

    # Identify duplicate/redundant tests
    print("\n📁 Directory Structure:")
    print("=" * 60)
    dirs = sorted(set(f.parent for cat_files in categories.values() for f in cat_files))
    for d in dirs[:20]:  # Show first 20
        print(f"  {d.relative_to(test_dir.parent)}")

    return categories


if __name__ == "__main__":
    analyze_tests()
