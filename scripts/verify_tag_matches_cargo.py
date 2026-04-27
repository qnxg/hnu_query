import sys
import tomllib
from pathlib import Path


def read_cargo_version(cargo_toml: Path) -> str:
    try:
        data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        raise RuntimeError(f"invalid Cargo.toml: {exc}") from exc

    version = data.get("package", {}).get("version")
    if not isinstance(version, str):
        raise RuntimeError("cannot find `[package].version` in Cargo.toml")
    return version
def normalize_tag(tag: str) -> str:
    tag = tag.strip()
    if tag.startswith("refs/tags/"):
        tag = tag.removeprefix("refs/tags/")
    if tag.startswith("v"):
        tag = tag[1:]
    return tag


def main() -> int:
    if len(sys.argv) != 2:
        print(
            "Usage: verify_tag_matches_cargo.py <tag>",
            file=sys.stderr,
        )
        return 2

    tag = normalize_tag(sys.argv[1])
    cargo_version = read_cargo_version(Path("Cargo.toml"))

    print(f"tag={tag} cargo={cargo_version}")
    if tag != cargo_version:
        print(
            "Tag version does not match Cargo.toml version.",
            file=sys.stderr,
        )
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

