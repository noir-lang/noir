#!/usr/bin/env bash

# Clear out the existing build artifacts as these aren't automatically removed by wasm-pack.
if [ -d ./pkg/ ]; then
    rm -rf ./pkg/
fi