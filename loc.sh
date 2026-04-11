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

workers="$(command -v nproc >/dev/null 2>&1 && nproc || getconf _NPROCESSORS_ONLN || echo 4)"
find "${targets[@]}" -type f \( -name '*.rs' -o -name '*.ron' -o -name '*.g' \) -print0 \
  | xargs -0 -n 1 -P "$workers" sh -c '
      f=$1
      lines=$(wc -l < "$f")
      words=$(tr "." " " < "$f" | wc -w)
      nonblank_chars=$(tr -d "[:space:]" < "$f" | wc -c)
      case "$f" in
        *.rs)  ext="rs" ;;
        *.ron) ext="ron" ;;
        *.g)   ext="tg" ;; # requested label
        *)     ext="other" ;;
      esac
      printf "%s\t%s\t%s\t%s\t%s\n" "$ext" "$lines" "$words" "$nonblank_chars" "1"
    ' sh >> "$tmp"

awk -F'\t' '
  {
    subtotal[$1] += $2
    word_subtotal[$1] += $3
    char_subtotal[$1] += $4
    file_subtotal[$1] += $5
    total += $2
    word_total += $3
    char_total += $4
    file_total += $5
  }
  END {
    printf "rs:  %d lines, %d words, %d files, %d non-blank chars\n", subtotal["rs"] + 0, word_subtotal["rs"] + 0, file_subtotal["rs"] + 0, char_subtotal["rs"] + 0
    printf "ron: %d lines, %d words, %d files, %d non-blank chars\n", subtotal["ron"] + 0, word_subtotal["ron"] + 0, file_subtotal["ron"] + 0, char_subtotal["ron"] + 0
    printf "tg:  %d lines, %d words, %d files, %d non-blank chars\n", subtotal["tg"] + 0, word_subtotal["tg"] + 0, file_subtotal["tg"] + 0, char_subtotal["tg"] + 0
    printf "----------------\n"
    printf "all: %d lines, %d words, %d files, %d non-blank chars\n", total + 0, word_total + 0, file_total + 0, char_total + 0
  }
' "$tmp"
