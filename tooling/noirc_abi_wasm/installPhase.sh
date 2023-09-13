#!/usr/bin/env bash
export self_path=$(dirname "$(readlink -f "$0")")

mkdir -p $out

cp ${self_path}/README.md ${self_path}/pkg/
cp ${self_path}/package.json ${self_path}/pkg/
cp -r ${self_path}/pkg/* $out/

echo "" >> $out/README.md
echo "## Tracking" >> $out/README.md
echo "Built from [noir-lang/noir@$GIT_COMMIT](https://github.com/noir-lang/noir/tree/$GIT_COMMIT)" >> $out/README.md

# Cleanup temporary pkg directory
rm -r $self_path/pkg
