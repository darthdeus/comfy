#!/bin/bash
set -euxo pipefail

for example in $(ls comfy/examples | grep -e "\.rs$" | sed "s/\.rs//"); do
  cargo run --example "$example" --features blobs
done
