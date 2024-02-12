import { AztecAddress, Contract, loadContractArtifact } from '@aztec/aztec.js';
import TokenContractJson from '@aztec/noir-contracts.js/artifacts/token_contract-Token' assert { type: 'json' };

import { readFileSync } from 'fs';

// docs:start:get-tokens
export async function getToken(client) {
  const addresses = JSON.parse(readFileSync('addresses.json'));
  return Contract.at(AztecAddress.fromString(addresses.token), loadContractArtifact(TokenContractJson), client);
}
// docs:end:get-tokens
