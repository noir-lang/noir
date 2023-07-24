#!/bin/bash
set -eu

#find ./cpp/src -type f -name "c_bind*.hpp" | ./scripts/decls_json.py > exports.json
cat ./scripts/c_bind_files.txt | ./scripts/decls_json.py > exports.json
(cd ./ts && yarn ts-node-esm ./src/bindgen/index.ts ../exports.json > ./src/barretenberg_api/index.ts)
