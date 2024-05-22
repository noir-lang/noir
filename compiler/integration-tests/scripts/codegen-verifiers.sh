#!/usr/bin/env bash
set -e

NARGO_BACKEND_PATH=${NARGO_BACKEND_PATH:-bb}

self_path=$(dirname "$(readlink -f "$0")")

repo_root=$self_path/../../..

# We want to move all the contracts to the root of compiler/integration-tests
contracts_dir=$self_path/../contracts
rm -rf $contracts_dir
mkdir $contracts_dir

KEYS=$(mktemp -d)

# Codegen verifier contract for 1_mul
mul_dir=$repo_root/test_programs/execution_success/1_mul
nargo --program-dir $mul_dir compile
$NARGO_BACKEND_PATH write_vk -b $mul_dir/target/1_mul.json -o $KEYS/1_mul 
$NARGO_BACKEND_PATH contract -k $KEYS/1_mul -o $contracts_dir/1_mul.sol 

# Codegen verifier contract for assert_statement
assert_statement_dir=$repo_root/test_programs/execution_success/assert_statement
nargo --program-dir $assert_statement_dir compile
$NARGO_BACKEND_PATH write_vk -b $assert_statement_dir/target/assert_statement.json -o $KEYS/assert_statement
$NARGO_BACKEND_PATH contract -k $KEYS/assert_statement -o $contracts_dir/assert_statement.sol

# Codegen verifier contract for recursion
recursion_dir=$repo_root/compiler/integration-tests/circuits/recursion
nargo --program-dir $recursion_dir compile
$NARGO_BACKEND_PATH write_vk -b $recursion_dir/target/recursion.json -o $KEYS/recursion
$NARGO_BACKEND_PATH contract -k $KEYS/recursion ./ -o $contracts_dir/recursion.sol

rm -rf $KEYS