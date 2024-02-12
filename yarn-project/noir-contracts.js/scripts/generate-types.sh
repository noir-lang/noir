#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="./src"
INDEX="$OUT_DIR/index.ts"

rm -rf $OUT_DIR && mkdir -p $OUT_DIR

#
if ! ls ../../noir-projects/noir-contracts/target/*.json >/dev/null 2>&1; then
  echo "Error: No .json files found in noir-contracts/target folder."
  echo "Make sure noir-contracts is built before running this script."
  exit 1
fi

# Generate index.ts header.
echo "// Auto generated module - do not edit!" >$INDEX

# Ensure the artifacts directory exists
mkdir -p artifacts

for ABI in $(find ../../noir-projects/noir-contracts/target -maxdepth 1 -type f ! -name 'debug_*' -name '*.json'); do
  # Extract the filename from the path
  filename=$(basename "$ABI")

  # Copy the JSON file to the artifacts folder
  cp "$ABI" "artifacts/$filename"

  # Generate the contract name for referencing in the codegen command and index
  CONTRACT=$(jq -r .name "artifacts/$filename")

  echo "Creating types for $CONTRACT using artifacts/$filename..."
  node --no-warnings ../noir-compiler/dest/cli.js codegen -o $OUT_DIR --ts "artifacts/$filename"

  # Add contract import/export to index.ts.
  echo "export * from './${CONTRACT}.js';" >>$INDEX
done

echo "Formatting..."
yarn formatting:fix
