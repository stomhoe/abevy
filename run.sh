#!/usr/bin/env bash
set -euo pipefail

setsid bash -c '
cargo run -r -p argentum_coop --bin argentum_coop &
child=$!
trap "killall -9 argentum_coop 2>/dev/null" INT
'
