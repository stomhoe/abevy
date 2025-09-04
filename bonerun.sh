#!/bin/bash

projectRoot="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$projectRoot" || exit 1

rm -f rustc*.txt

cargo run > >(tee onerun_out.txt) 2>onerun_err.txt
