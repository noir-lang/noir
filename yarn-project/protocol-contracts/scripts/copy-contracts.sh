#! /bin/bash
set -euo pipefail
mkdir -p ./src/artifacts

contracts=(
  contract_class_registerer_contract-ContractClassRegisterer
  contract_instance_deployer_contract-ContractInstanceDeployer
  gas_token_contract-GasToken
  multi_call_entrypoint_contract-MultiCallEntrypoint
)

for contract in "${contracts[@]}"; do
  cp "../../noir-projects/noir-contracts/target/$contract.json" ./src/artifacts/${contract#*-}.json
done
