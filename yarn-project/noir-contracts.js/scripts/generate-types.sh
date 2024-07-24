#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="./src"
INDEX="$OUT_DIR/index.ts"

mkdir -p $OUT_DIR

# Check for .json files existence
if ! ls ../../noir-projects/noir-contracts/target/*.json >/dev/null 2>&1; then
  echo "Error: No .json files found in noir-contracts/target folder."
  echo "Make sure noir-contracts is built before running this script."
  exit 1
fi

# Generate index.ts header
echo "// Auto generated module - do not edit!" >"$INDEX"

# Ensure the artifacts directory exists
mkdir -p artifacts

decl=$(cat <<EOF
import { type NoirCompiledContract } from '@aztec/types/noir';
const circuit: NoirCompiledContract;
export = circuit;
EOF
);

for ABI in $(find ../../noir-projects/noir-contracts/target -maxdepth 1 -type f ! -name 'debug_*' -name '*.json'); do
  # Extract the filename from the path
  filename=$(basename "$ABI")
  dts_file=$(echo $filename | sed 's/.json/.d.json.ts/g');

  # Copy the JSON file to the artifacts folder
  cp "$ABI" "artifacts/$filename"
  echo "$decl" > "artifacts/$dts_file"
done

# Generate types for the contracts
node --no-warnings ../builder/dest/bin/cli.js codegen -o $OUT_DIR artifacts

# Append exports for each generated TypeScript file to index.ts
find "$OUT_DIR" -maxdepth 1 -type f -name '*.ts' ! -name 'index.ts' | while read -r TS_FILE; do
  CONTRACT_NAME=$(basename "$TS_FILE" .ts) # Remove the .ts extension to get the contract name
  echo "export * from './${CONTRACT_NAME}.js';" >>"$INDEX"
done
