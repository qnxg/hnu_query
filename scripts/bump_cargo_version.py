import re
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: bump_cargo_version.py <version>", file=sys.stderr)
        return 2

    version = sys.argv[1].strip()
    if not version or version.startswith("v"):
        print(f"Invalid version: {version!r}", file=sys.stderr)
        return 2

    path = Path("Cargo.toml")
    text = path.read_text(encoding="utf-8")

    package_match = re.search(
        r"(?ms)^(\[package\]\s*\n)(.*?)(?=^\[|\Z)",
        text,
    )
    if package_match is None:
        print("Failed to find [package] section in Cargo.toml", file=sys.stderr)
        return 1

    package_body = package_match.group(2)
    new_package_body, n = re.subn(
        r'(?m)^version\s*=\s*"[^"]+"\s*$',
        f'version = "{version}"',
        package_body,
    )
    if n != 1:
        print(
            "Failed to update exactly one version entry in [package] section of Cargo.toml",
            file=sys.stderr,
        )
        return 1

    new_text = (
        text[: package_match.start(2)]
        + new_package_body
        + text[package_match.end(2) :]
    )
    path.write_text(new_text, encoding="utf-8")
    print(f"Updated Cargo.toml version -> {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

