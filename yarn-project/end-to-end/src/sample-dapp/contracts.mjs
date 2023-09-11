import { Contract } from '@aztec/aztec.js';
import {
  PrivateTokenContractAbi as PrivateTokenArtifact,
  PublicTokenContractAbi as PublicTokenArtifact,
} from '@aztec/noir-contracts/artifacts';

import { readFileSync } from 'fs';

// docs:start:get-tokens
export async function getPrivateToken(client) {
  const addresses = JSON.parse(readFileSync('addresses.json'));
  return Contract.at(addresses.privateToken, PrivateTokenArtifact, client);
}

export async function getPublicToken(client) {
  const addresses = JSON.parse(readFileSync('addresses.json'));
  return Contract.at(addresses.publicToken, PublicTokenArtifact, client);
}
// docs:end:get-tokens
