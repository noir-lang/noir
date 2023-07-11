#!/usr/bin/env bash

mkdir -p $out
cp ./crates/wasm/README.md $out/
cp ./crates/wasm/package.json $out/
cp -r ./pkg/* $out/

echo "## Tracking" >> $out/README.md
echo "Built from [noir-lang/noir@$GIT_COMMIT](https://github.com/noir-lang/noir/tree/$GIT_COMMIT)" >> $out/README.md
