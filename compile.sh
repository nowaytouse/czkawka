#!/bin/bash
# Pure compile script for Czkawka GUI (Release Mode)
echo "Starting compilation of czkawka_gui in release mode..."
cargo build --release --bin czkawka_gui
if [ $? -eq 0 ]; then
    echo "Compilation successful! Binary located at target/release/czkawka_gui"
else
    echo "Compilation failed."
    exit 1
fi
