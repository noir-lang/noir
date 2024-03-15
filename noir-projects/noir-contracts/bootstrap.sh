#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

CMD=${1:-}

if [ -n "$CMD" ]; then
  if [ "$CMD" = "clean" ]; then
    git clean -fdx
    exit 0
  else
    echo "Unknown command: $CMD"
    exit 1
  fi
fi

echo "Compiling contracts..."
NARGO=${NARGO:-../../noir/noir-repo/target/release/nargo}
$NARGO compile --silence-warnings

echo "Transpiling avm contracts..."
for contract_json in target/avm_test_*.json; do
  echo Transpiling $contract_json...
  TRANSPILER=${TRANSPILER:-../../avm-transpiler/target/release/avm-transpiler}
  $TRANSPILER $contract_json $contract_json
done