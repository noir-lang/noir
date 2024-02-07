import { ContractArtifact } from '@aztec/foundation/abi';
import { loadContractArtifact } from '@aztec/types/abi';
import { NoirCompiledContract } from '@aztec/types/noir';

import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

// Copied from the build output for the contract `Benchmarking` in noir-contracts
export function getSampleContractArtifact(): ContractArtifact {
  const path = getPathToFixture('Benchmarking.test.json');
  const content = JSON.parse(readFileSync(path).toString()) as NoirCompiledContract;
  return loadContractArtifact(content);
}

// Copied from the test 'registers a new contract class' in end-to-end/src/e2e_deploy_contract.test.ts
export function getSampleContractClassRegisteredEventPayload(): Buffer {
  const path = getPathToFixture('ContractClassRegisteredEventData.hex');
  return Buffer.from(readFileSync(path).toString(), 'hex');
}

// Copied from the test 'deploying a contract instance' in end-to-end/src/e2e_deploy_contract.test.ts
export function getSampleContractInstanceDeployedEventPayload(): Buffer {
  const path = getPathToFixture('ContractInstanceDeployedEventData.hex');
  return Buffer.from(readFileSync(path).toString(), 'hex');
}

function getPathToFixture(name: string) {
  return resolve(dirname(fileURLToPath(import.meta.url)), `../../fixtures/${name}`);
}
