#!/bin/bash
set -euo pipefail;

ROOT=$(pwd)
for CONTRACT_NAME in "$@"; do
  CONTRACT_FOLDER="${CONTRACT_NAME}_contract"
  echo "Compiling $CONTRACT_NAME..."
  cd src/contracts/$CONTRACT_FOLDER
  rm -f target/*
  if [[ -z "${VERBOSE:-}" ]]; then
    nargo compile main --contracts 2> /dev/null > /dev/null
  else
    nargo compile main --contracts
  fi

  # Trim the output of the noir json, removing unneeded fields to make it small enough to be processed by copy_output.ts
  # This is a workaround until noir output files get smaller
  FILENAME=$(ls target/*.json)
  echo "Trimming output for $FILENAME"
  jq -c '.functions[] |= del(.proving_key, .verification_key)' $FILENAME > temp.json 
  mv temp.json $FILENAME

  cd $ROOT
  echo "Copying output for $CONTRACT_NAME"
  NODE_OPTIONS=--no-warnings yarn ts-node --esm src/scripts/copy_output.ts $CONTRACT_NAME
  echo -e "Done\n"
done