#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

CMD=${1:-}

if [ -n "$CMD" ]; then
  if [ "$CMD" = "clean" ]; then
    git clean -fdx
    exit 0
  else
    echo "Unknown command: $CMD"
    exit 1
  fi
fi

rm -f build-wasm/CMakeCache.txt

# Build WASM.
if [ -n "${WASM_DEBUG:-}" ] ; then
  cmake --preset wasm-dbg
  cmake --build --preset wasm-dbg --target aztec3-circuits.wasm
else
  cmake --preset wasm
  cmake --build --preset wasm --target aztec3-circuits.wasm
fi
