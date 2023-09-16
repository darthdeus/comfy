#!/bin/bash
set -euxo pipefail

declare -a examples=(sprite)

for example in "${examples[@]}"; do
  cargo build --example "$example" --release --target wasm32-unknown-unknown
done
