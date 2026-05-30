#!/usr/bin/env python3
"""Response Time Analysis Script for ZEN-MCP-Server.

Automated API response time measurement and reporting.
"""

import argparse
import json
import statistics
import time
from typing import Any

import requests


class ResponseTimeAnalyzer:
    """
    Analyzes API response times and performance metrics.
    """

    def __init__(self, base_url: str = "http://localhost:8000"):
        self.base_url = base_url
        self.response_times = []
        self.percentiles = [50, 90, 95, 99]

    def measure_endpoint(self, endpoint: str, method: str = "GET", **kwargs) -> dict[str, Any]:
        """
        Measure response time for a specific endpoint.
        """
        url = f"{self.base_url}{endpoint}"
        start_time = time.time()

        try:
            if method.upper() == "GET":
                response = requests.get(url, timeout=30, **kwargs)
            elif method.upper() == "POST":
                response = requests.post(url, timeout=30, **kwargs)
            else:
                response = requests.request(method, url, timeout=30, **kwargs)

            end_time = time.time()
            response_time = (end_time - start_time) * 1000  # Convert to milliseconds

            measurement = {
                "endpoint": endpoint,
                "method": method,
                "url": url,
                "response_time_ms": response_time,
                "status_code": response.status_code,
                "timestamp": start_time,
                "success": 200 <= response.status_code < 300,
            }

            self.response_times.append(measurement)
            return measurement

        except requests.exceptions.RequestException as e:
            end_time = time.time()
            response_time = (end_time - start_time) * 1000

            measurement = {
                "endpoint": endpoint,
                "method": method,
                "url": url,
                "response_time_ms": response_time,
                "status_code": 0,
                "timestamp": start_time,
                "success": False,
                "error": str(e),
            }

            self.response_times.append(measurement)
            return measurement

    def measure_multiple_endpoints(self, endpoints: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """
        Measure response times for multiple endpoints.
        """
        results = []
        for endpoint_config in endpoints:
            endpoint = endpoint_config.get("endpoint", "")
            method = endpoint_config.get("method", "GET")
            kwargs = endpoint_config.get("kwargs", {})

            result = self.measure_endpoint(endpoint, method, **kwargs)
            results.append(result)

        return results

    def calculate_statistics(self) -> dict[str, Any]:
        """
        Calculate response time statistics.
        """
        if not self.response_times:
            return {}

        successful_times = [rt["response_time_ms"] for rt in self.response_times if rt["success"]]
        all_times = [rt["response_time_ms"] for rt in self.response_times]

        if not successful_times:
            return {"error": "No successful requests"}

        stats = {
            "total_requests": len(self.response_times),
            "successful_requests": len(successful_times),
            "failed_requests": len(self.response_times) - len(successful_times),
            "success_rate": len(successful_times) / len(self.response_times) * 100,
            "average_response_time": statistics.mean(successful_times),
            "median_response_time": statistics.median(successful_times),
            "min_response_time": min(successful_times),
            "max_response_time": max(successful_times),
            "standard_deviation": (
                statistics.stdev(successful_times) if len(successful_times) > 1 else 0
            ),
        }

        # Calculate percentiles
        for percentile in self.percentiles:
            stats[f"p{percentile}_response_time"] = self._calculate_percentile(
                successful_times, percentile,
            )

        return stats

    def _calculate_percentile(self, data: list[float], percentile: float) -> float:
        """
        Calculate percentile value.
        """
        sorted_data = sorted(data)
        index = (percentile / 100) * (len(sorted_data) - 1)
        if index.is_integer():
            return sorted_data[int(index)]
        lower = sorted_data[int(index)]
        upper = sorted_data[int(index) + 1]
        return lower + (upper - lower) * (index - int(index))

    def identify_slow_endpoints(self, threshold_ms: float = 1000) -> list[dict[str, Any]]:
        """
        Identify endpoints with response times above threshold.
        """
        return [
            rt
            for rt in self.response_times
            if rt["success"] and rt["response_time_ms"] > threshold_ms
        ]

    def identify_failed_endpoints(self) -> list[dict[str, Any]]:
        """
        Identify failed endpoints.
        """
        return [rt for rt in self.response_times if not rt["success"]]

    def generate_report(self) -> str:
        """
        Generate comprehensive response time report.
        """
        stats = self.calculate_statistics()
        slow_endpoints = self.identify_slow_endpoints()
        failed_endpoints = self.identify_failed_endpoints()

        report = []
        report.append("ZEN-MCP-Server Response Time Analysis Report")
        report.append("=" * 50)

        if "error" in stats:
            report.append(f"Error: {stats['error']}")
            return "\n".join(report)

        report.append(f"Total Requests: {stats['total_requests']}")
        report.append(f"Successful Requests: {stats['successful_requests']}")
        report.append(f"Failed Requests: {stats['failed_requests']}")
        report.append(f"Success Rate: {stats['success_rate']:.2f}%")
        report.append(f"Average Response Time: {stats['average_response_time']:.2f}ms")
        report.append(f"Median Response Time: {stats['median_response_time']:.2f}ms")
        report.append(f"Min Response Time: {stats['min_response_time']:.2f}ms")
        report.append(f"Max Response Time: {stats['max_response_time']:.2f}ms")
        report.append(f"Standard Deviation: {stats['standard_deviation']:.2f}ms")

        report.append("\nPercentiles:")
        for percentile in self.percentiles:
            report.append(f"  P{percentile}: {stats[f'p{percentile}_response_time']:.2f}ms")

        if slow_endpoints:
            report.append(f"\nSlow Endpoints (>1000ms): {len(slow_endpoints)}")
            for endpoint in slow_endpoints[:5]:  # Show top 5
                report.append(f"  {endpoint['endpoint']}: {endpoint['response_time_ms']:.2f}ms")

        if failed_endpoints:
            report.append(f"\nFailed Endpoints: {len(failed_endpoints)}")
            for endpoint in failed_endpoints[:5]:  # Show top 5
                error_msg = endpoint.get("error", "Unknown error")
                report.append(f"  {endpoint['endpoint']}: {error_msg}")

        return "\n".join(report)


def main():
    """
    Main response time analysis function.
    """
    parser = argparse.ArgumentParser(description="Analyze API response times")
    parser.add_argument("--url", default="http://localhost:8000", help="Base URL for API")
    parser.add_argument("--endpoints", nargs="+", help="Endpoints to test")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")

    args = parser.parse_args()

    analyzer = ResponseTimeAnalyzer(args.url)

    # Default endpoints if none specified
    if not args.endpoints:
        args.endpoints = ["/health", "/api/v1/status", "/mcp/ping", "/metrics"]

    # Measure response times
    for endpoint in args.endpoints:
        analyzer.measure_endpoint(endpoint)

    if args.report:
        report = analyzer.generate_report()
        print(report)
        return 0

    stats = analyzer.calculate_statistics()

    if args.json:
        print(
            json.dumps({"statistics": stats, "response_times": analyzer.response_times}, indent=2),
        )
    else:
        print("Response Time Analysis Results:")
        if "error" in stats:
            print(f"  Error: {stats['error']}")
        else:
            print(f"  Success Rate: {stats.get('success_rate', 0):.2f}%")
            print(f"  Average Response Time: {stats.get('average_response_time', 0):.2f}ms")
            print(f"  P95 Response Time: {stats.get('p95_response_time', 0):.2f}ms")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
