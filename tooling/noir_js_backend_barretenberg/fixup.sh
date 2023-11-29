#!/usr/bin/env bash

# Put these package.json files in the cjs and
# mjs directory respectively, so that 
# tools can recognise that the .js files are either
# commonjs or ESM files.
self_path=$(dirname "$(readlink -f "$0")")

cjs_package='{
    "type": "commonjs"
}'

esm_package='{
    "type": "module"
}'

echo "$cjs_package" > $self_path/lib/cjs/package.json
echo "$esm_package" > $self_path/lib/esm/package.json