#!/usr/bin/env python3
"""Publish a crate to crates.io only when that exact version is not published yet."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request
from pathlib import Path


def package_name_and_version(manifest_dir: Path) -> tuple[str, str]:
    cargo_toml = manifest_dir / "Cargo.toml"
    try:
        data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        raise RuntimeError(f"invalid {cargo_toml}: {exc}") from exc

    package = data.get("package", {})
    name = package.get("name")
    version = package.get("version")
    if not isinstance(name, str) or not isinstance(version, str):
        raise RuntimeError(f"cannot find `[package].name` / `.version` in {cargo_toml}")
    return name, version


def is_published_with_curl(url: str) -> bool | None:
    curl = shutil.which("curl")
    if curl is None:
        return None

    # -f: 404 等非 2xx 视为失败；-sS: 静默但保留错误
    result = subprocess.run(
        [
            curl,
            "-fsS",
            "-o",
            os.devnull,
            "-H",
            "User-Agent: hnu_query-release",
            url,
        ],
        check=False,
    )
    if result.returncode == 0:
        return True
    if result.returncode == 22:
        return False
    raise RuntimeError(f"curl failed with exit code {result.returncode}")


def is_published_with_urllib(url: str) -> bool:
    request = urllib.request.Request(url, headers={"User-Agent": "hnu_query-release"})
    try:
        with urllib.request.urlopen(request) as response:
            return response.status == 200
    except urllib.error.HTTPError as exc:
        if exc.code == 404:
            return False
        raise
    except urllib.error.URLError as exc:
        reason = exc.reason
        if "CERTIFICATE_VERIFY_FAILED" in str(reason):
            raise RuntimeError(
                "HTTPS 证书校验失败。当前 Python 可能未正确配置 CA（例如 Inkscape 自带的 Python）。"
                "请改用系统 Python（`py` / `python3`），或安装/使用带 curl 的环境；"
                "GitHub Actions 上不受影响。"
            ) from exc
        raise


def is_published(name: str, version: str) -> bool:
    url = f"https://crates.io/api/v1/crates/{name}/{version}"
    published = is_published_with_curl(url)
    if published is not None:
        return published
    return is_published_with_urllib(url)


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: cargo_publish_if_new.py <crate-directory>", file=sys.stderr)
        return 2

    manifest_dir = Path(sys.argv[1])
    name, version = package_name_and_version(manifest_dir)

    if is_published(name, version):
        print(f"{name} {version} already on crates.io, skipping publish")
        return 0

    print(f"Publishing {name} {version}...")
    return subprocess.call(["cargo", "publish"], cwd=manifest_dir)


if __name__ == "__main__":
    raise SystemExit(main())
