#! /bin/bash
set -euo pipefail
mkdir -p ./fixtures

cp "../../noir-projects/noir-contracts/target/benchmarking_contract-Benchmarking.json" ./fixtures/Benchmarking.test.json
