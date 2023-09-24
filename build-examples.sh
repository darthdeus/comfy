#!/bin/bash
set -euxo pipefail

mkdir -p target/generated/
rm -rf target/generated/*

EXAMPLE=sprite

for example in $(echo $EXAMPLE); do
  RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown --release --example "$example" --features blobs
  # cp -r examples/$1/resources target/generated/ || true
  dir="target/generated/$example"
  mkdir -p "$dir"
  sed "s/{{example}}/$example/" > "$dir/index.html" < index.html
  wasm-bindgen --out-dir "$dir" --target web "target/wasm32-unknown-unknown/release/examples/$example.wasm"
done
