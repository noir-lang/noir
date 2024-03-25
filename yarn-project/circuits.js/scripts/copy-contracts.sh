#! /bin/bash
set -euo pipefail
mkdir -p ./fixtures

cp "../../noir-projects/noir-contracts/target/benchmarking_contract-Benchmarking.json" ./fixtures/Benchmarking.test.json
cp "../../noir-projects/noir-contracts/target/test_contract-Test.json" ./fixtures/Test.test.json
