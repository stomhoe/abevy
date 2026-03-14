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
      nonblank_chars=$(tr -d '[:space:]' < "$f" | wc -c)
      case "$f" in
        *.rs)  ext="rs" ;;
        *.ron) ext="ron" ;;
        *.g)   ext="tg" ;; # requested label
        *)     ext="other" ;;
      esac
      printf "%s\t%s\t%s\n" "$ext" "$lines" "$nonblank_chars" >> "$tmp"
    done

awk -F'\t' '
  {
    subtotal[$1] += $2
    char_subtotal[$1] += $3
    total += $2
    char_total += $3
  }
  END {
    printf "rs:  %d lines, %d non-blank chars\n", subtotal["rs"] + 0, char_subtotal["rs"] + 0
    printf "ron: %d lines, %d non-blank chars\n", subtotal["ron"] + 0, char_subtotal["ron"] + 0
    printf "tg:  %d lines, %d non-blank chars\n", subtotal["tg"] + 0, char_subtotal["tg"] + 0
    printf "----------------\n"
    printf "all: %d lines, %d non-blank chars\n", total + 0, char_total + 0
  }
' "$tmp"
