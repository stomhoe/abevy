#!/usr/bin/env bash
set -euo pipefail

# Count LOC in: crates/, assets/, and main.rs
# Subtotals by extension: .rs, .ron, .g (shown as "tg")

targets=()
[[ -d crates ]] && targets+=("crates")
[[ -d assets ]] && targets+=("assets")
[[ -f main.rs ]] && targets+=("main.rs")

if [[ ${#targets[@]} -eq 0 ]]; then
  echo "No target paths found (expected crates/, assets/, or main.rs)"
  exit 1
fi

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

find "${targets[@]}" -type f \( -name '*.rs' -o -name '*.ron' -o -name '*.g' \) -print0 \
  | while IFS= read -r -d '' f; do
      lines=$(wc -l < "$f")
      case "$f" in
        *.rs)  ext="rs" ;;
        *.ron) ext="ron" ;;
        *.g)   ext="tg" ;; # requested label
        *)     ext="other" ;;
      esac
      printf "%s\t%s\n" "$ext" "$lines" >> "$tmp"
    done

awk -F'\t' '
  {
    subtotal[$1] += $2
    total += $2
  }
  END {
    printf "rs:  %d\n", subtotal["rs"] + 0
    printf "ron: %d\n", subtotal["ron"] + 0
    printf "tg:  %d\n", subtotal["tg"] + 0
    printf "----------------\n"
    printf "all: %d\n", total + 0
  }
' "$tmp"
