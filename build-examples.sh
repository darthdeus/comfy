#!/bin/bash
set -euxo pipefail

for example in comfy/examples/*.rs; do
  cargo build --example "$(basename -s .rs "${example}")" --features blobs,ldtk
done
