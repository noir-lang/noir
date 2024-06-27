#! /bin/bash
set -euo pipefail
mkdir -p ./artifacts

contracts=(
  contract_class_registerer_contract-ContractClassRegisterer
  contract_instance_deployer_contract-ContractInstanceDeployer
  gas_token_contract-GasToken
  key_registry_contract-KeyRegistry
  auth_registry_contract-AuthRegistry
  multi_call_entrypoint_contract-MultiCallEntrypoint
)


decl=$(cat <<EOF
import { type NoirCompiledContract } from '@aztec/types/noir';
const circuit: NoirCompiledContract;
export = circuit;
EOF
);

for contract in "${contracts[@]}"; do
  cp "../../noir-projects/noir-contracts/target/$contract.json" ./artifacts/${contract#*-}.json
  echo "$decl" > ./artifacts/${contract#*-}.d.json.ts
done
