import { type ContractArtifact } from '@aztec/foundation/abi';

import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

import { loadContractArtifact } from '../abi/contract_artifact.js';
import { type NoirCompiledContract } from '../noir/index.js';

// Copied from the build output for the contract `Benchmarking` in noir-contracts
export function getSampleContractArtifact(): ContractArtifact {
  const path = getPathToFixture('Benchmarking.test.json');
  const content = JSON.parse(readFileSync(path).toString()) as NoirCompiledContract;
  return loadContractArtifact(content);
}

function getPathToFixture(name: string) {
  return resolve(dirname(fileURLToPath(import.meta.url)), `../../fixtures/${name}`);
}
