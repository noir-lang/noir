#! /bin/bash
set -euo pipefail
mkdir -p ./src/artifacts

contracts=(contract_class_registerer_contract-ContractClassRegisterer contract_instance_deployer_contract-ContractInstanceDeployer)

for contract in "${contracts[@]}"; do
  cp "../noir-contracts/target/$contract.json" ./src/artifacts/${contract#*-}.json
done

yarn run -T prettier -w ./src/artifacts