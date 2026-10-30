#!/bin/bash
set -euo pipefail

rm -rf flatpak
generator_dir=$(mktemp -d "${TMPDIR:-/tmp}/czkawka-flatpak-generator.XXXXXX")
trap 'rm -rf "$generator_dir"' EXIT
wget -O "$generator_dir/flatpak-cargo-generator.py" https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
mkdir flatpak
uv run --no-project --upgrade --prerelease disallow --script "$generator_dir/flatpak-cargo-generator.py" ./Cargo.lock -o flatpak/cargo-sources.json
