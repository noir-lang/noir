#!/usr/bin/env bash

mkdir -p $out
cp crates/$pname/README.md $out/
cp crates/$pname/package.json $out/
cp -r ./pkg/* $out/

echo "## Tracking" >> $out/README.md
echo "Built from [noir-lang/noir@$GIT_COMMIT](https://github.com/noir-lang/noir/tree/$GIT_COMMIT)" >> $out/README.md
