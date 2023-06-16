#!/usr/bin/env bash

# TODO: Handle the wasm target being built in release mode
WASM_BINARY=./target/wasm32-unknown-unknown/release/${pname}.wasm
NODE_WASM=./pkg/nodejs/${pname}_bg.wasm
BROWSER_WASM=./pkg/web/${pname}_bg.wasm

wasm-bindgen $WASM_BINARY --out-dir ./pkg/nodejs --typescript --target nodejs
wasm-bindgen $WASM_BINARY --out-dir ./pkg/web --typescript --target web
wasm-opt $NODE_WASM -o $NODE_WASM -O
wasm-opt $BROWSER_WASM -o $BROWSER_WASM -O
