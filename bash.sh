#!/usr/bin/env bash

set -e

# Go to project root (this script should be there)
cd "$(dirname "$0")"

# Step 1: Build the project
cargo build

# Step 2: Start two cargo run processes and capture their output
cargo run > run1_out.txt 2> run1_err.txt &
pid1=$!
cargo run > run2_out.txt 2> run2_err.txt &
pid2=$!

echo "Both cargo runs started. Press Ctrl+C to stop them."

# Trap Ctrl+C and kill both processes
stopped=0
cleanup() {
    if [[ $stopped -eq 0 ]]; then
        stopped=1
        echo "Stopping both cargo processes..."
        kill "$pid1" "$pid2" 2>/dev/null || true
        wait "$pid1" "$pid2" 2>/dev/null
    fi
}
trap cleanup SIGINT SIGTERM

# Wait for both processes to exit or for Ctrl+C
wait "$pid1"
wait "$pid2"
