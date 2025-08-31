#!/usr/bin/env bash

# Default names if no arguments are given
NAMES=("tilemap" "common" "dimension" "game_common")

# Use arguments if provided
if [ "$#" -gt 0 ]; then
    NAMES=("$@")
fi

# Always include argentum_coop
NAMES+=("argentum_coop")

# Build prefix list (with and without 'lib')
PREFIXES=()
for name in "${NAMES[@]}"; do
    if [[ "$name" == lib* ]]; then
        PREFIXES+=("$name")
        PREFIXES+=("${name:3}")
    else
        PREFIXES+=("$name")
        PREFIXES+=("lib$name")
    fi
done

# Build regex pattern for matching
PATTERN="($(IFS="|"; echo "${PREFIXES[*]}"))"
REGEX=".*/${PATTERN}.*"

echo "Deleting build artifacts matching: ${PREFIXES[*]}"
echo "Using regex: $REGEX"

# Remove files in target/debug
if [ -d "target/debug" ]; then
    find target/debug -type f -regextype posix-extended -regex "$REGEX" -print -exec rm -f {} +
fi

# Remove directories in target/debug/incremental
INC_PATH="target/debug/incremental"
if [ -d "$INC_PATH" ]; then
    find "$INC_PATH" -mindepth 1 -maxdepth 1 -type d -regextype posix-extended -regex "$REGEX" -print -exec rm -rf {} +
fi