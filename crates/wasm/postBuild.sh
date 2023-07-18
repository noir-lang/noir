#!/usr/bin/env bash

# TODO: Handle the wasm target being built in release mode
WASM_BINARY=./target/wasm32-unknown-unknown/release/${pname}.wasm
NODE_DIR=./pkg/nodejs
BROWSER_DIR=./pkg/web
NODE_WASM=${NODE_DIR}/${pname}_bg.wasm
BROWSER_WASM=${BROWSER_DIR}/${pname}_bg.wasm

wasm-bindgen $WASM_BINARY --out-dir ${NODE_DIR} --typescript --target nodejs
wasm-bindgen $WASM_BINARY --out-dir ${BROWSER_DIR} --typescript --target web
wasm-opt $NODE_WASM -o $NODE_WASM -O
wasm-opt $BROWSER_WASM -o $BROWSER_WASM -O
