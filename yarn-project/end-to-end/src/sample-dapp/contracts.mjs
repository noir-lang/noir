import { Contract } from '@aztec/aztec.js';
import { TokenContractAbi } from '@aztec/noir-contracts/artifacts';

import { readFileSync } from 'fs';

// docs:start:get-tokens
export async function getToken(client) {
  const addresses = JSON.parse(readFileSync('addresses.json'));
  return Contract.at(addresses.token, TokenContractAbi, client);
}
// docs:end:get-tokens
