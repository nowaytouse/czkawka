#!/bin/bash
# Script to build Czkawka GUI in release mode
cd "$(dirname "$0")/.."
cargo build --release --bin czkawka_gui
