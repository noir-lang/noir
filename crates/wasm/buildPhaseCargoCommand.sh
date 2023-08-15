#!/usr/bin/env bash

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

export self_path=$(dirname "$(readlink -f "$0")")

# Clear out the existing build artifacts as these aren't automatically removed by wasm-pack.
if [ -d ${self_path}/pkg/ ]; then
    rm -rf ${self_path}/pkg/
fi

# Check that the user passed in debug or release mode
# and set the BUILD_FLAG and BUILD_MODE appropriately.
if [[ -z "$1" ]]; then
    echo "Build script requires either "debug" or "release" as an argument."
    exit 1
fi

BUILD_MODE=$1
BUILD_FLAG="" # Defaults to debug mode which is an empty string 

if [[ "$1" == "release" ]]; then
    BUILD_MODE=release
    BUILD_FLAG="--release"
elif [[ "$1" != "debug" ]]; then
    echo "Invalid BUILD_MODE. Accepted values are 'debug' or 'release'."
    exit 1
fi

TARGET=wasm32-unknown-unknown
WASM_BINARY=${CARGO_TARGET_DIR}/${TARGET}/${BUILD_MODE}/${pname}.wasm

NODE_DIR=${self_path}/pkg/nodejs/
BROWSER_DIR=${self_path}/pkg/web/
NODE_WASM=${NODE_DIR}/${pname}_bg.wasm
BROWSER_WASM=${BROWSER_DIR}/${pname}_bg.wasm

# Build the new wasm package
run_or_fail cargo build --lib $BUILD_FLAG --package wasm --target wasm32-unknown-unknown
run_or_fail wasm-bindgen $WASM_BINARY --out-dir $NODE_DIR --typescript --target nodejs
run_or_fail wasm-bindgen $WASM_BINARY --out-dir $BROWSER_DIR --typescript --target web
run_if_available wasm-opt $NODE_WASM -o $NODE_WASM -O
run_if_available wasm-opt $BROWSER_WASM -o $BROWSER_WASM -O
