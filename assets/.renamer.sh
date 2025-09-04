#!/bin/bash

find . -type f | while read -r file; do
    newfile="$file"
    newfile="$(echo "$newfile" | sed -e 's/_left/_west/g' -e 's/_right/_east/g' -e 's/_up/_north/g' -e 's/_down/_south/g')"
    if [[ "$file" != "$newfile" ]]; then
        mv "$file" "$newfile"
    fi
done
