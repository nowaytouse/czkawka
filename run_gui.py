#!/usr/bin/env python3
"""Selectable launcher for Czkawka (GTK) or Krokiet (Slint). Skips recompile if binary is up-to-date."""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent

APPS = {
    "1": ("krokiet", "krokiet"),
    "2": ("czkawka_gui", "czkawka_gui"),
}


def _paths_newer_than(bin_mtime: float, paths: list[Path]) -> bool:
    for path in paths:
        if not path.exists():
            continue
        if path.is_file():
            if path.stat().st_mtime > bin_mtime:
                return True
            continue
        for f in path.rglob("*"):
            if f.is_file() and f.stat().st_mtime > bin_mtime:
                return True
    return False


def binary_is_fresh(bin_path: Path, crate: str) -> bool:
    if not bin_path.exists():
        return False
    bin_mtime = bin_path.stat().st_mtime
    watch_paths = [
        ROOT / crate / "src",
        ROOT / crate / "ui",
        ROOT / crate / "Cargo.toml",
        ROOT / crate / "build.rs",
        ROOT / "czkawka_core" / "src",
        ROOT / "czkawka_core" / "Cargo.toml",
        ROOT / "Cargo.toml",
        ROOT / "Cargo.lock",
    ]
    return not _paths_newer_than(bin_mtime, watch_paths)


def main() -> None:
    print("Select app to run:")
    print("  1) Krokiet (Slint frontend)")
    print("  2) Czkawka (GTK frontend)")
    choice = input("Choice [1]: ").strip() or "1"

    if choice not in APPS:
        print(f"Invalid choice: {choice}", file=sys.stderr)
        sys.exit(1)

    crate, bin_name = APPS[choice]
    bin_path = ROOT / "target" / "release" / bin_name

    if binary_is_fresh(bin_path, crate):
        print(f"Binary {bin_path} is up-to-date, skipping recompile.")
        subprocess.run([str(bin_path)], cwd=ROOT, check=True)
    else:
        subprocess.run(
            ["cargo", "run", "--release", "--bin", bin_name],
            cwd=ROOT,
            check=True,
        )


if __name__ == "__main__":
    main()
