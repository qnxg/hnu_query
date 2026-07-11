#!/usr/bin/env python3
"""Generate a public API diff with cargo-public-api.

Requires:
  - cargo-public-api on PATH (`cargo install cargo-public-api`)
  - a recent nightly toolchain (`rustup install nightly --profile minimal`)

Usage:
  python scripts/public_api_diff.py
  python scripts/public_api_diff.py main
  python scripts/public_api_diff.py origin/main..HEAD
  python scripts/public_api_diff.py --output api-diff.txt main
  python scripts/public_api_diff.py --simplified origin/main..HEAD
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def resolve_diff_spec(spec: str | None) -> str:
    if spec is None:
        return "origin/main..HEAD"
    if ".." in spec:
        return spec
    return f"{spec}..HEAD"


def build_command(diff_spec: str, simplified: bool) -> list[str]:
    command = ["cargo", "public-api", "diff", diff_spec]
    if simplified:
        # Equivalent to omitting blanket / auto-trait / auto-derived impls.
        command.append("-sss")
    return command


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Diff the crate public API between two git refs.",
    )
    parser.add_argument(
        "diff",
        nargs="?",
        default=None,
        help="Base ref, or `base..head` (default: origin/main..HEAD)",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=None,
        help="Write diff text to this file (also printed to stdout)",
    )
    parser.add_argument(
        "--simplified",
        action="store_true",
        help="Omit noisy blanket / auto-trait / auto-derived impls",
    )
    args = parser.parse_args(argv)

    diff_spec = resolve_diff_spec(args.diff)
    command = build_command(diff_spec, args.simplified)

    print(f"+ {' '.join(command)}", file=sys.stderr)
    completed = subprocess.run(
        command,
        check=False,
        capture_output=True,
        text=True,
    )

    # cargo-public-api prints the human-readable diff on stdout.
    # It may exit non-zero when the API changed; that is expected for reporting.
    output = completed.stdout
    if completed.stderr:
        print(completed.stderr, end="", file=sys.stderr)

    if completed.returncode not in (0, 1):
        print(
            f"cargo public-api failed with exit code {completed.returncode}",
            file=sys.stderr,
        )
        if output:
            print(output, end="")
        return completed.returncode

    if not output.strip():
        output = "No public API changes detected.\n"

    print(output, end="")
    if args.output is not None:
        args.output.write_text(output, encoding="utf-8")
        print(f"Wrote {args.output}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
