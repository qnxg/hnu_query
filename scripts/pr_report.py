#!/usr/bin/env python3
"""Build a markdown PR report from coverage and public API diff artifacts.

Usage:
  python scripts/pr_report.py \\
    --lcov lcov.info \\
    --api-diff api-diff.txt \\
    --base origin/main \\
    --output pr-report.md

  python scripts/pr_report.py \\
    --lcov lcov.info \\
    --api-diff api-diff.txt \\
    --base origin/main \\
    --codecov-url https://app.codecov.io/gh/owner/repo/pull/123 \\
    --output pr-report.md
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

REPORT_MARKER = "<!-- hnu-query-ci-report -->"


@dataclass(frozen=True)
class CoverageSummary:
    lines_found: int
    lines_hit: int

    @property
    def percent(self) -> float:
        if self.lines_found == 0:
            return 0.0
        return 100.0 * self.lines_hit / self.lines_found


def normalize_path(path: str) -> str:
    return path.replace("\\", "/").removeprefix("./")


def parse_lcov_files(path: Path) -> dict[str, CoverageSummary]:
    """Parse lcov into a map of normalized relative path -> per-file summary."""
    files: dict[str, CoverageSummary] = {}
    current_file: str | None = None
    lines_found = 0
    lines_hit = 0

    def flush() -> None:
        nonlocal current_file, lines_found, lines_hit
        if current_file is not None:
            files[current_file] = CoverageSummary(
                lines_found=lines_found,
                lines_hit=lines_hit,
            )
        current_file = None
        lines_found = 0
        lines_hit = 0

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if raw_line.startswith("SF:"):
            flush()
            current_file = normalize_path(raw_line.removeprefix("SF:"))
        elif raw_line.startswith("LF:"):
            lines_found = int(raw_line.removeprefix("LF:"))
        elif raw_line.startswith("LH:"):
            lines_hit = int(raw_line.removeprefix("LH:"))
        elif raw_line == "end_of_record":
            flush()

    flush()
    return files


def total_coverage(files: dict[str, CoverageSummary]) -> CoverageSummary:
    return CoverageSummary(
        lines_found=sum(item.lines_found for item in files.values()),
        lines_hit=sum(item.lines_hit for item in files.values()),
    )


def changed_files_vs_base(base: str) -> list[str]:
    """Return paths changed between merge-base(base, HEAD) and HEAD."""
    completed = subprocess.run(
        ["git", "diff", "--name-only", "--diff-filter=ACMR", f"{base}...HEAD"],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        stderr = completed.stderr.strip() or "unknown git error"
        raise RuntimeError(f"git diff failed: {stderr}")

    files: list[str] = []
    for line in completed.stdout.splitlines():
        path = normalize_path(line.strip())
        if path:
            files.append(path)
    return files


def coverage_for_changed_files(
    coverage_by_file: dict[str, CoverageSummary],
    changed_files: list[str],
) -> list[tuple[str, CoverageSummary]]:
    """Keep only changed files that appear in the coverage report."""
    rows: list[tuple[str, CoverageSummary]] = []
    for path in changed_files:
        summary = coverage_by_file.get(path)
        if summary is None:
            continue
        rows.append((path, summary))
    rows.sort(key=lambda item: (item[1].percent, item[0]))
    return rows


def read_optional_text(path: Path | None) -> str:
    if path is None:
        return ""
    if not path.is_file():
        raise FileNotFoundError(f"file not found: {path}")
    return path.read_text(encoding="utf-8").strip()


def render_coverage_section(
    summary: CoverageSummary | None,
    changed_coverage: list[tuple[str, CoverageSummary]],
    codecov_url: str | None,
) -> list[str]:
    lines = ["### Code Coverage", ""]

    if summary is None:
        lines.append("_Coverage data unavailable._")
        lines.append("")
        return lines

    lines.extend(
        [
            "| Metric | Value |",
            "| --- | ---: |",
            f"| Lines found | {summary.lines_found} |",
            f"| Lines hit | {summary.lines_hit} |",
            f"| Line coverage | {summary.percent:.2f}% |",
            "",
        ]
    )

    lines.append("#### Changed files")
    lines.append("")
    if not changed_coverage:
        lines.append(
            "_No changed files with coverage data "
            "(relative to the comparison base)._"
        )
        lines.append("")
    else:
        lines.extend(
            [
                "| File | Hit | Found | Coverage |",
                "| --- | ---: | ---: | ---: |",
            ]
        )
        for path, file_summary in changed_coverage:
            lines.append(
                f"| `{path}` | {file_summary.lines_hit} | "
                f"{file_summary.lines_found} | {file_summary.percent:.2f}% |"
            )
        lines.append("")

    if codecov_url:
        lines.append(f"[View full report on Codecov]({codecov_url})")
        lines.append("")

    return lines


def render_api_section(api_diff: str) -> list[str]:
    lines = ["### Public API Changes", ""]

    if not api_diff or api_diff == "No public API changes detected.":
        lines.append("No public API changes detected.")
        lines.append("")
        return lines

    # Keep PR comments readable; truncate very large diffs.
    max_chars = 60_000
    body = api_diff
    truncated = False
    if len(body) > max_chars:
        body = body[:max_chars].rstrip() + "\n\n… (truncated)"
        truncated = True

    lines.append("<details>")
    lines.append("<summary>Show public API diff</summary>")
    lines.append("")
    lines.append("```diff")
    lines.append(body)
    lines.append("```")
    if truncated:
        lines.append("")
        lines.append("_Diff truncated for GitHub comment size limits._")
    lines.append("")
    lines.append("</details>")
    lines.append("")
    return lines


def build_report(
    summary: CoverageSummary | None,
    changed_coverage: list[tuple[str, CoverageSummary]],
    api_diff: str,
    codecov_url: str | None,
) -> str:
    parts = [
        REPORT_MARKER,
        "## CI Report",
        "",
        *render_coverage_section(summary, changed_coverage, codecov_url),
        *render_api_section(api_diff),
    ]
    return "\n".join(parts).rstrip() + "\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Generate a markdown report for PR comments.",
    )
    parser.add_argument(
        "--lcov",
        type=Path,
        default=None,
        help="Path to lcov.info produced by scripts/coverage.py",
    )
    parser.add_argument(
        "--api-diff",
        type=Path,
        default=None,
        help="Path to public API diff text from scripts/public_api_diff.py",
    )
    parser.add_argument(
        "--base",
        default="origin/main",
        help="Git ref to compare against for changed files (default: origin/main)",
    )
    parser.add_argument(
        "--codecov-url",
        default=None,
        help="Optional Codecov URL to link from the report",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        required=True,
        help="Write markdown report to this path",
    )
    args = parser.parse_args(argv)

    coverage_by_file: dict[str, CoverageSummary] = {}
    summary: CoverageSummary | None = None
    if args.lcov is not None:
        if not args.lcov.is_file():
            print(f"lcov file not found: {args.lcov}", file=sys.stderr)
            return 1
        coverage_by_file = parse_lcov_files(args.lcov)
        summary = total_coverage(coverage_by_file)

    try:
        changed = changed_files_vs_base(args.base)
    except RuntimeError as exc:
        print(exc, file=sys.stderr)
        return 1

    changed_coverage = coverage_for_changed_files(coverage_by_file, changed)
    print(
        f"Changed files: {len(changed)}; "
        f"with coverage: {len(changed_coverage)}",
        file=sys.stderr,
    )

    try:
        api_diff = read_optional_text(args.api_diff)
    except FileNotFoundError as exc:
        print(exc, file=sys.stderr)
        return 1

    report = build_report(summary, changed_coverage, api_diff, args.codecov_url)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(report, encoding="utf-8")
    print(f"Wrote {args.output}", file=sys.stderr)
    print(report, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
