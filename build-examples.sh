#!/bin/bash
set -euxo pipefail

mkdir -p target/generated/
rm -rf target/generated/*

template="""
+++
title = "Particles" 
description = "" 
date = 2019-11-27

[extra] 
screenshot = "/comfy.png" 
gh_source = "//github.com/not-fl3/macroquad/blob/master/examples/particles_example.rs" 
wasm_source = "/gen_examples/particles_particles.html" 
+++
"""

for example in $(cat examples.txt); do
  RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown --release --example "$example"
  # cp -r examples/$1/resources target/generated/ || true
  dir="target/generated/$example"
  mkdir -p "$dir"
  sed "s/{{example}}/$example/" > "$dir/index.html" < index.html
  wasm-bindgen --out-dir "$dir" --target web "target/wasm32-unknown-unknown/release/examples/$example.wasm"
  echo "$template"
done
