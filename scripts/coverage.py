#!/usr/bin/env python3
"""Run code coverage with configurable ignore rules.

Usage:
  python scripts/coverage.py --html
  python scripts/coverage.py --open
  python scripts/coverage.py --text --summary-only
  python scripts/coverage.py --lcov --output-path lcov.info
  python scripts/coverage.py --list-rules

Edit COVERAGE_IGNORE_REGEXES and COVERAGE_IGNORE_FILES below to adjust exclusions.
"""

from __future__ import annotations

import re
import subprocess
import sys

# Regex rules merged into one `--ignore-filename-regex` for cargo llvm-cov.
COVERAGE_IGNORE_REGEXES: list[str] = [
    r"[/\\](fetch|login|raw)\.rs$",
    r"[/\\]mod\.rs$",
    r"[/\\]test\.rs$",
]

# Specific files to exclude (paths relative to the current working directory).
COVERAGE_IGNORE_FILES: list[str] = [
    "src/cas/tfa.rs",
    "src/error.rs",
    "src/test/obs.rs",
    "src/utils/obs.rs"
]


def path_to_ignore_regex(rel_path: str) -> str:
    normalized = rel_path.replace("\\", "/").removeprefix("./")
    escaped = "".join(
        r"[/\\]" if char == "/" else re.escape(char) for char in normalized
    )
    return f"{escaped}$"


def collect_ignore_patterns() -> list[str]:
    patterns: list[str] = []

    for pattern in COVERAGE_IGNORE_REGEXES:
        if pattern:
            patterns.append(pattern)

    for file_path in COVERAGE_IGNORE_FILES:
        if file_path:
            patterns.append(path_to_ignore_regex(file_path))

    return patterns


def build_combined_ignore_regex(patterns: list[str]) -> str:
    # llvm-cov's regex engine does not handle (?:...) reliably; join with | only.
    return "|".join(patterns)


def build_ignore_args() -> list[str]:
    patterns = collect_ignore_patterns()
    if not patterns:
        return []
    return [
        "--ignore-filename-regex",
        build_combined_ignore_regex(patterns),
    ]


def build_base_args() -> list[str]:
    # Show paths relative to the workspace root instead of absolute paths.
    return ["--remap-path-prefix", *build_ignore_args()]


def print_rules() -> None:
    patterns = collect_ignore_patterns()

    print("Coverage ignore rules:")
    print()

    print("Regex rules:")
    if not COVERAGE_IGNORE_REGEXES:
        print("  (none)")
    else:
        for pattern in COVERAGE_IGNORE_REGEXES:
            if pattern:
                print(f"  - {pattern}")

    print()
    print("Specific files:")
    if not COVERAGE_IGNORE_FILES:
        print("  (none)")
    else:
        for file_path in COVERAGE_IGNORE_FILES:
            if not file_path:
                continue
            regex = path_to_ignore_regex(file_path)
            print(f"  - {file_path}")
            print(f"    => {regex}")

    print()
    print("Combined regex passed to cargo llvm-cov:")
    if not patterns:
        print("  (none)")
    else:
        print(f"  {build_combined_ignore_regex(patterns)}")


def main(argv: list[str]) -> int:
    if argv[:1] == ["--list-rules"]:
        print_rules()
        return 0

    command = ["cargo", "llvm-cov", *build_base_args(), *argv]
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
