#!/usr/bin/env bash

function require_command {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is required but not installed." >&2
        exit 1
    fi
}
function run_or_fail {
  "$@"
  local status=$?
  if [ $status -ne 0 ]; then
    echo "Command '$*' failed with exit code $status" >&2
    exit $status
  fi
}
function run_if_available {
  if command -v "$1" >/dev/null 2>&1; then
    "$@"
  else
    echo "$1 is not installed. Please install it to use this feature." >&2
  fi
}

require_command jq
require_command cargo
require_command wasm-bindgen
require_command wasm-opt

self_path=$(dirname "$(readlink -f "$0")")
pname=$(cargo read-manifest | jq -r '.name')

NODE_DIR=$self_path/nodejs
BROWSER_DIR=$self_path/web

# Clear out the existing build artifacts as these aren't automatically removed by wasm-bindgen.
if [ -d ./pkg/ ]; then
    rm -r $NODE_DIR
    rm -r $BROWSER_DIR
fi

TARGET=wasm32-unknown-unknown
WASM_BINARY=${self_path}/../../target/$TARGET/release/${pname}.wasm

NODE_WASM=${NODE_DIR}/${pname}_bg.wasm
BROWSER_WASM=${BROWSER_DIR}/${pname}_bg.wasm

# Build the new wasm package
run_or_fail cargo build --lib --release --target $TARGET --package ${pname}
run_or_fail wasm-bindgen $WASM_BINARY --out-dir $NODE_DIR --typescript --target nodejs
run_or_fail wasm-bindgen $WASM_BINARY --out-dir $BROWSER_DIR --typescript --target web
run_if_available wasm-opt $NODE_WASM -o $NODE_WASM -O
run_if_available wasm-opt $BROWSER_WASM -o $BROWSER_WASM -O
