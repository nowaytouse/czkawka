#!/bin/bash
# Pure run script for Czkawka GUI
# This script executes the pre-compiled release binary directly.

BINARY="target/release/czkawka_gui"

if [ -f "$BINARY" ]; then
    echo "Running $BINARY..."
    ./"$BINARY" "$@"
else
    echo "Error: Binary not found at $BINARY"
    echo "Please run ./compile.sh first to build the application."
    exit 1
fi
