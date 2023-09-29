#!/bin/bash

cd "$(dirname "$0")"

# Run codegen-verifier for 1_mul
cd tooling/nargo_cli/tests/execution_success/1_mul
nargo codegen-verifier

# Run codegen-verifier for main
cd compiler/integration-tests/test/circuits/main
nargo codegen-verifier

# Copy compiled contracts from the root of compiler/integration-tests
cp tooling/nargo_cli/tests/execution_success/1_mul/contract/1_mul/plonk_vk.sol foundry-project/src/1_mul.sol
cp compiler/integration-tests/test/circuits/main/contract/main/plonk_vk.sol foundry-project/src/main.sol
