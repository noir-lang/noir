#!/usr/bin/env bash
set -euo pipefail

CONTRACTS=(
  test_contract-Test
)

NOIR_CONTRACTS_DIR="../../noir-projects/noir-contracts"

for contract in $CONTRACTS; do
  echo "Generating Noir interface for $contract"
  OUT_DIR="$NOIR_CONTRACTS_DIR/contracts/${contract%%-*}/src"
  node --no-warnings ../noir-compiler/dest/cli.js codegen -o $OUT_DIR --nr --force $NOIR_CONTRACTS_DIR/target/$contract.json
  mv $OUT_DIR/${contract#*-}.nr $OUT_DIR/interface.nr
done

echo "Done updating Noir interfaces"