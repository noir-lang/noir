#!/bin/bash
set -euo pipefail;

ROOT=$(pwd)
echo "Using $(nargo --version)"
for CONTRACT_NAME in "$@"; do
  CONTRACT_FOLDER="${CONTRACT_NAME}_contract"
  echo "Compiling $CONTRACT_NAME..."
  cd src/contracts/$CONTRACT_FOLDER
  rm -f target/*
  if [[ -z "${VERBOSE:-}" ]]; then
    nargo compile main --experimental-ssa --contracts 2> /dev/null > /dev/null  || (echo "Error compiling contract. Re-running as verbose to show compiler output:"; nargo compile main --experimental-ssa --contracts);
  else
    nargo compile main --experimental-ssa --contracts
  fi

  cd $ROOT
  echo "Copying output for $CONTRACT_NAME"
  NODE_OPTIONS=--no-warnings yarn ts-node --esm src/scripts/copy_output.ts $CONTRACT_NAME
  
  echo "Formatting contract folders"
  yarn run -T prettier -w ./src/artifacts/$CONTRACT_FOLDER.json ../aztec.js/src/abis/*.json ./src/types/*.ts
  echo -e "Done\n"
  
done