#!/bin/sh
./src/wasi-sdk-20.0/bin/llvm-strip ./build-wasm/bin/barretenberg.wasm
./src/wasi-sdk-20.0/bin/llvm-strip ./build-wasm-threads/bin/barretenberg.wasm
