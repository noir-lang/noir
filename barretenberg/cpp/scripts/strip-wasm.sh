#!/bin/sh
/opt/wasi-sdk/bin/llvm-strip ./build-wasm/bin/barretenberg.wasm
# TODO(https://github.com/AztecProtocol/barretenberg/issues/941) We currently do not strip barretenberg threaded wasm, for stack traces.
# /opt/wasi-sdk/bin/llvm-strip ./build-wasm-threads/bin/barretenberg.wasm
