#!/usr/bin/env bash

self_path=$(dirname "$(readlink -f "$0")")

repo_root=$self_path/../../..

# Run codegen-verifier for 1_mul
mul_dir=$repo_root/test_programs/execution_success/1_mul
nargo --program-dir $mul_dir codegen-verifier

# Run codegen-verifier for assert_statement
assert_statement_dir=$repo_root/test_programs/execution_success/assert_statement
nargo --program-dir $assert_statement_dir codegen-verifier

# Run codegen-verifier for recursion
recursion_dir=$repo_root/compiler/integration-tests/circuits/recursion
nargo --program-dir $recursion_dir codegen-verifier

# Copy compiled contracts from the root of compiler/integration-tests
contracts_dir=$self_path/../contracts
rm -rf $contracts_dir
mkdir $contracts_dir

cp $mul_dir/contract/1_mul/plonk_vk.sol $contracts_dir/1_mul.sol
cp $assert_statement_dir/contract/assert_statement/plonk_vk.sol $contracts_dir/assert_statement.sol
cp $recursion_dir/contract/recursion/plonk_vk.sol $contracts_dir/recursion.sol
