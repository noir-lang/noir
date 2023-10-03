#!/bin/bash

self_path=$(dirname "$(readlink -f "$0")")

repo_root=$self_path/../../..

# Run codegen-verifier for 1_mul
mul_dir=$repo_root/tooling/nargo_cli/tests/execution_success/1_mul
nargo --program-dir $mul_dir codegen-verifier

# Run codegen-verifier for main
main_dir=$repo_root/compiler/integration-tests/test/circuits/main
nargo --program-dir $main_dir codegen-verifier

# Copy compiled contracts from the root of compiler/integration-tests
src_dir=$self_path/../foundry-project/src
cp $mul_dir/contract/1_mul/plonk_vk.sol $src_dir/1_mul.sol
cp $main_dir/contract/main/plonk_vk.sol $src_dir/main.sol
