#!/bin/sh
set -e

if [ -z "$SKIP_CPP_BUILD" ]; then
  # Build the wasms and strip debug symbols.
  cd ../cpp
  cmake --preset wasm-threads && cmake --build --preset wasm-threads
  cmake --preset wasm && cmake --build --preset wasm
  ./scripts/strip-wasm.sh
  cd ../ts
fi

# Copy the wasm to its home in the bb.js dest folder.
# We only need the threads wasm, as node always uses threads.
# We need to take two copies for both esm and cjs builds. You can't use symlinks when publishing.
# This probably isn't a big deal however due to compression.
# When building the the browser bundle, both wasms are inlined directly.
mkdir -p ./dest/node/barretenberg_wasm
mkdir -p ./dest/node-cjs/barretenberg_wasm
cp ../cpp/build-wasm-threads/bin/barretenberg.wasm ./dest/node/barretenberg_wasm/barretenberg-threads.wasm
cp ../cpp/build-wasm-threads/bin/barretenberg.wasm ./dest/node-cjs/barretenberg_wasm/barretenberg-threads.wasm