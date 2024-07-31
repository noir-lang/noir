#!/usr/bin/env bash
# Run from aztec-packages/barretenberg.
set -eu

if ! dpkg -l python3-clang-18 &> /dev/null; then
  echo "You need to install python clang 18 e.g.: apt install python3-clang-18"
  exit 1
fi

#find ./cpp/src -type f -name "c_bind*.hpp" > ./scripts/c_bind_files.txt
cat ./scripts/c_bind_files.txt | ./scripts/decls_json.py > exports.json
(
  cd ./ts && \
  yarn install && \
  yarn node --loader ts-node/esm ./src/bindgen/index.ts ../exports.json > ./src/barretenberg_api/index.ts && \
  yarn prettier -w ./src/barretenberg_api/index.ts
)
