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

    new_text, n = re.subn(
        r'(?m)^version\s*=\s*"[^"]+"\s*$',
        f'version = "{version}"',
        text,
        count=1,
    )
    if n != 1:
        print("Failed to update version in Cargo.toml", file=sys.stderr)
        return 1

    path.write_text(new_text, encoding="utf-8")
    print(f"Updated Cargo.toml version -> {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

