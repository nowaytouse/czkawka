#!/bin/bash
# Script to run pre-built Czkawka GUI binary
cd "$(dirname "$0")/.."
if [ -f "target/release/czkawka_gui" ]; then
    ./target/release/czkawka_gui "$@"
else
    echo "Error: Binary not found. Please run scripts/build.sh first."
    exit 1
fi
