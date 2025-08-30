#!/usr/bin/env bash

set -e

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$project_root"

# Remove rustc*.txt files
rm -f rustc*.txt

# Step 1: Build the project
cargo build

# Step 2: Start two cargo processes and capture their output without warnings
cargo run --quiet > run1_out.txt 2> run1_err.txt &
pid1=$!
cargo run --quiet > run2_out.txt 2> run2_err.txt &
pid2=$!

# Also stream their stdout to the current process
tail -f run1_out.txt &
tail1_pid=$!
tail -f run2_out.txt &
tail2_pid=$!

echo "Both cargo runs started. Press Ctrl+C to stop them."

# Trap Ctrl+C and kill both processes
cleanup() {
    kill $pid1 $pid2 $tail1_pid $tail2_pid 2>/dev/null || true
    wait $pid1 $pid2 2>/dev/null || true
    exit
}
trap cleanup SIGINT SIGTERM

# Wait for both processes to exit
wait $pid1 $pid2

# Cleanup tails
kill $tail1_pid $tail2_pid 2>/dev/null || true