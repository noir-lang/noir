
#!/bin/bash

# Example:
# - this script will automatically be run when running `yarn noir:build`
# - it exists on its own to allow ci to compile and format in different contexts, as the noir compiler is not available in yarn project base ( by choice )
# - you can run `yarn noir:types:all` to create all noir artifacts and types consumed by aztec packages.

# Enable strict mode:
# Exit on error (set -e), treat unset variables as an error (set -u),
set -eu;

artifacts_dir="src/artifacts"
types_dir="src/types"

# Create output directories
mkdir -p $types_dir
mkdir -p $artifacts_dir


ROOT=$(pwd)

write_import() {
    CONTRACT_NAME=$1
    NAME=$(echo $CONTRACT_NAME | perl -pe 's/(^|_)(\w)/\U$2/g')

    echo "import ${NAME}Json from './${CONTRACT_NAME}_contract.json' assert { type: 'json' };"  >> "$artifacts_dir/index.ts";
}

write_export() {
    CONTRACT_NAME=$1
    NAME=$(echo $CONTRACT_NAME | perl -pe 's/(^|_)(\w)/\U$2/g')

    # artifacts
    echo "export const ${NAME}ContractAbi = ${NAME}Json as ContractAbi;"  >> "$artifacts_dir/index.ts";
    echo "Written typescript for $NAME"

    # types
    echo "export * from './${CONTRACT_NAME}.js';" >> "$types_dir/index.ts";
}


process() {
  CONTRACT=$1

  cd $ROOT
  echo "Creating types for $CONTRACT"
  NODE_OPTIONS=--no-warnings yarn ts-node --esm src/scripts/copy_output.ts $CONTRACT_NAME
}

format(){
  echo "Formatting contract folders"
  yarn run -T prettier -w  ../aztec.js/src/abis/*.json ./$types_dir/*.ts
  echo -e "Done\n"
}

# Make type files
for CONTRACT_NAME in "$@"; do
  process $CONTRACT_NAME &
done

# Wait for all background processes to finish
wait

# Write the index ts stuff
# Remove the output file
rm -f $artifacts_dir/index.ts || true

# Generate artifacts package index.ts
echo "// Auto generated module\n" > "$artifacts_dir/index.ts";
echo "import { ContractAbi } from '@aztec/foundation/abi';"  >> "$artifacts_dir/index.ts";

# Generate types package index.ts
echo "// Auto generated module\n" > "$types_dir/index.ts";
for CONTRACT_NAME in "$@"; do
    write_import $CONTRACT_NAME
    write_export $CONTRACT_NAME
done

# only run the rest when the full flag is set
format
