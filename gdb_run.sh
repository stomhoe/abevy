#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

SYSROOT="$(rustc --print sysroot)"
export LD_LIBRARY_PATH="$ROOT_DIR/target/debug/deps:$SYSROOT/lib:$SYSROOT/lib/rustlib/x86_64-unknown-linux-gnu/lib:${LD_LIBRARY_PATH:-}"
export BEVY_ASSET_ROOT="$ROOT_DIR"

RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes -Awarnings" cargo build --bin argentum_coop

gdb -q \
  -ex "set pagination off" \
  -ex "run" \
  ./target/debug/argentum_coop
