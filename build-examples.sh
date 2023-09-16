#!/bin/bash
set -euxo pipefail

declare -a examples=(
  animated_shapes
  animated_sprites
  particles
  post_processing
  shapes
  sprite
  text
  timed_draw
)

mkdir -p target/generated/
rm -rf target/generated/*

for example in "${examples[@]}"; do
  RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown --release --example "$example"
  # cp -r examples/$1/resources target/generated/ || true
  dir="target/generated/$example"
  sed "s/{{example}}/$example/" > "$dir/index.html" < index.html
  wasm-bindgen --out-dir "$dir" --target web "target/wasm32-unknown-unknown/release/examples/$example.wasm"
done
