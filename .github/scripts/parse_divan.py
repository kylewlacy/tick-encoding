#!/usr/bin/env python3
"""Parse Divan benchmark output to JSON format.

Divan (https://github.com/nvzqz/divan) outputs benchmark results in a
tree-table format. This script converts that output to JSON for further
processing, such as benchmark comparisons in CI pipelines.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# Matches time values like "1.816 ms", "710.1 µs", "500 ns"
TIME_PATTERN = re.compile(r"([\d.]+)\s*(ns|µs|us|ms|s)")

# Matches lines starting with tree-drawing characters (├, │, └, ╰, ─)
TREE_LINE_PATTERN = re.compile(r"^\s*[├│└╰─]")

# Matches benchmark data lines, including nested ones like "│  ├─ 16   <data...>"
BENCH_LINE_PATTERN = re.compile(r"^\s*(?:│\s*)*[├└╰]─\s*(\S+)\s+(.*)")

# Splits on column separators (│) with optional surrounding whitespace
COLUMN_SPLIT_PATTERN = re.compile(r"\s*│\s*")


@dataclass
class TimeValue:
    """Represents a time measurement with value in nanoseconds."""

    value: int | None
    unit: str = "ns"

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {"value": self.value, "unit": self.unit}


@dataclass
class BenchmarkResult:
    """Represents a single benchmark result with all timing metrics."""

    name: str
    fastest: TimeValue
    slowest: TimeValue
    median: TimeValue
    mean: TimeValue
    samples: int | None
    iters: int | None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "name": self.name,
            "fastest": self.fastest.to_dict(),
            "slowest": self.slowest.to_dict(),
            "median": self.median.to_dict(),
            "mean": self.mean.to_dict(),
            "samples": self.samples,
            "iters": self.iters,
        }

    def validate(self) -> list[str]:
        """Validate benchmark result for consistency.

        Returns a list of warning messages. Empty list means valid.
        """
        warnings: list[str] = []

        # Check timing consistency: fastest <= median <= slowest
        if (
            self.fastest.value is not None
            and self.median.value is not None
            and self.slowest.value is not None
        ):
            if self.fastest.value > self.median.value:
                warnings.append(
                    f"{self.name}: fastest ({self.fastest.value}) > "
                    f"median ({self.median.value})"
                )
            if self.median.value > self.slowest.value:
                warnings.append(
                    f"{self.name}: median ({self.median.value}) > "
                    f"slowest ({self.slowest.value})"
                )

        # Check samples is positive
        if self.samples is not None and self.samples <= 0:
            warnings.append(
                f"{self.name}: samples must be positive, got {self.samples}"
            )

        # Check iters is positive
        if self.iters is not None and self.iters <= 0:
            warnings.append(f"{self.name}: iters must be positive, got {self.iters}")

        return warnings


def parse_time_to_ns(time_str: str) -> int | None:
    """Convert a time string to nanoseconds.

    Accepts formats like '1.816 ms', '710.1 µs', '500 ns', '1.5 s'.

    Args:
        time_str: Time string to parse

    Returns:
        Time value in nanoseconds, or None if parsing fails
    """
    time_str = time_str.strip()
    if not time_str:
        return None

    match = TIME_PATTERN.match(time_str)
    if not match:
        return None

    value = float(match.group(1))
    unit = match.group(2)

    multipliers: dict[str, int] = {
        "ns": 1,
        "µs": 1_000,
        "us": 1_000,
        "ms": 1_000_000,
        "s": 1_000_000_000,
    }

    return int(value * multipliers.get(unit, 1))


def parse_int(value_str: str) -> int | None:
    """Parse an integer from a string.

    Args:
        value_str: String to parse

    Returns:
        Parsed integer, or None if parsing fails
    """
    value_str = value_str.strip()
    if not value_str:
        return None
    try:
        return int(value_str)
    except ValueError:
        return None


def parse_line(
    line: str, current_group: str, current_subgroup: str
) -> BenchmarkResult | str | None:
    """Parse a single benchmark data line.

    Args:
        line: The line to parse (should match BENCH_LINE_PATTERN)
        current_group: The current benchmark group name
        current_subgroup: The current benchmark subgroup name (for nested benchmarks)

    Returns:
        BenchmarkResult if line contains valid benchmark data,
        str (new subgroup name) if line is a subgroup header,
        None if line doesn't match.
    """
    match = BENCH_LINE_PATTERN.match(line)
    if not match:
        return None

    bench_name = match.group(1)
    rest = match.group(2)

    # Split by │ to get timing columns
    columns = COLUMN_SPLIT_PATTERN.split(rest)
    if len(columns) < 6:
        return None

    # Columns are: fastest, slowest, median, mean, samples, iters
    fastest_ns = parse_time_to_ns(columns[0])
    slowest_ns = parse_time_to_ns(columns[1])
    median_ns = parse_time_to_ns(columns[2])
    mean_ns = parse_time_to_ns(columns[3])
    samples = parse_int(columns[4])
    iters = parse_int(columns[5])

    # If no timing data, this is a subgroup header
    if mean_ns is None:
        return bench_name

    # Build full benchmark name with optional subgroup
    if current_subgroup:
        full_name = f"{current_group}/{current_subgroup}/{bench_name}"
    elif current_group:
        full_name = f"{current_group}/{bench_name}"
    else:
        full_name = bench_name

    return BenchmarkResult(
        name=full_name,
        fastest=TimeValue(fastest_ns),
        slowest=TimeValue(slowest_ns),
        median=TimeValue(median_ns),
        mean=TimeValue(mean_ns),
        samples=samples,
        iters=iters,
    )


def parse_divan_output(content: str) -> list[BenchmarkResult]:
    """Parse Divan benchmark output and return list of benchmark results.

    This is a pure function with no side effects.

    Args:
        content: Raw Divan benchmark output text

    Returns:
        List of BenchmarkResult objects
    """
    results: list[BenchmarkResult] = []
    current_group = ""
    current_subgroup = ""

    for line in content.splitlines():
        # Skip empty lines
        if not line.strip():
            continue

        # Check for header lines (contain column names: fastest, slowest, median)
        # Extract group name from the beginning of the header before skipping
        if "fastest" in line and "slowest" in line and "median" in line:
            parts = COLUMN_SPLIT_PATTERN.split(line)
            if parts:
                # The group name is at the start of the line, before "fastest"
                first_part = parts[0].strip()
                words = first_part.split()
                # Filter out "fastest" if it's in the first part
                group_words = [w for w in words if w != "fastest"]
                if group_words:
                    current_group = group_words[0]
                    current_subgroup = ""  # Reset subgroup on new group
            continue

        # Check for group header (line without tree characters)
        # Group headers are lines that don't start with tree chars (├, │, └)
        if not TREE_LINE_PATTERN.match(line):
            # Check if this is a group name (no timing data)
            parts = COLUMN_SPLIT_PATTERN.split(line)
            if len(parts) == 1 or (len(parts) > 1 and not parse_time_to_ns(parts[1])):
                # This might be a group header
                stripped = line.strip()
                if stripped and not stripped.startswith("Timer precision"):
                    current_group = stripped.split()[0] if stripped.split() else ""
                    current_subgroup = ""  # Reset subgroup on new group
                continue
            continue

        # Parse benchmark line with tree structure
        # Check if this is a nested line (starts with │) or top-level
        is_nested = line.lstrip().startswith("│")
        if not is_nested:
            # Top-level benchmark line, reset subgroup
            current_subgroup = ""

        result = parse_line(line, current_group, current_subgroup)
        if isinstance(result, str):
            # Got a subgroup name
            current_subgroup = result
        elif isinstance(result, BenchmarkResult):
            results.append(result)

    return results


def read_input(path: str) -> str:
    """Read input from file or stdin.

    Args:
        path: File path, or '-' to read from stdin

    Returns:
        File contents as string
    """
    if path == "-":
        return sys.stdin.read()
    return Path(path).read_text()


def write_output(content: str, path: str | None) -> None:
    """Write output to file or stdout.

    Args:
        content: Content to write
        path: File path, or None to write to stdout
    """
    if path is None:
        sys.stdout.write(content)
    else:
        Path(path).write_text(content)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Parse Divan benchmark output to JSON format.",
    )
    parser.add_argument(
        "input",
        help="Input file path, or '-' to read from stdin",
    )
    parser.add_argument(
        "-o",
        "--output",
        help="Output file path (default: stdout)",
    )

    args = parser.parse_args()

    # Read input
    try:
        content = read_input(args.input)
    except FileNotFoundError:
        print(f"Error: File '{args.input}' not found", file=sys.stderr)
        sys.exit(1)
    except OSError as e:
        print(f"Error: Failed to read '{args.input}': {e}", file=sys.stderr)
        sys.exit(1)

    # Parse benchmark output
    results = parse_divan_output(content)

    # Convert to JSON
    json_data = [result.to_dict() for result in results]
    json_output = json.dumps(json_data, indent=2) + "\n"

    # Write output
    try:
        write_output(json_output, args.output)
    except OSError as e:
        print(f"Error: Failed to write output: {e}", file=sys.stderr)
        sys.exit(1)
