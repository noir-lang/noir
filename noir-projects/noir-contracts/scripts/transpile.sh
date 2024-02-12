#!/usr/bin/env bash
set -euo pipefail

echo "Transpiling contracts..."
for contract_json in target/avm_test_*.json; do
  echo Transpiling $contract_json...
  ../../avm-transpiler/target/release/avm-transpiler $contract_json $contract_json
done
