#!/usr/bin/env bash

mkdir -p $out
cp README.md $out/
cp ./crates/wasm/package.json $out/
cp -r ./pkg/* $out/