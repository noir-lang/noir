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

# Codegen verifier contract for a_1_mul
mul_dir=$repo_root/test_programs/execution_success/a_1_mul
nargo --program-dir $mul_dir compile --pedantic-solving
$NARGO_BACKEND_PATH write_vk -b $mul_dir/target/a_1_mul.json -o $KEYS --oracle_hash keccak
$NARGO_BACKEND_PATH write_solidity_verifier -k $KEYS/vk -o $contracts_dir/a_1_mul.sol

# Codegen verifier contract for assert_statement
assert_statement_dir=$repo_root/test_programs/execution_success/assert_statement
nargo --program-dir $assert_statement_dir compile --pedantic-solving
$NARGO_BACKEND_PATH write_vk -b $assert_statement_dir/target/assert_statement.json -o $KEYS --oracle_hash keccak
$NARGO_BACKEND_PATH write_solidity_verifier -k $KEYS/vk -o $contracts_dir/assert_statement.sol

# Codegen verifier contract for recursion
recursion_dir=$repo_root/compiler/integration-tests/circuits/recursion
nargo --program-dir $recursion_dir compile --pedantic-solving
$NARGO_BACKEND_PATH write_vk --scheme ultra_honk --oracle_hash keccak -b $recursion_dir/target/recursion.json -o $KEYS
$NARGO_BACKEND_PATH write_solidity_verifier --scheme ultra_honk -k $KEYS/vk -o $contracts_dir/recursion.sol

rm -rf $KEYS
