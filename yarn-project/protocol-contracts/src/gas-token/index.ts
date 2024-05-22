import { AztecAddress, GAS_TOKEN_ADDRESS } from '@aztec/circuits.js';

import { type ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { GasTokenArtifact } from './artifact.js';

/** Returns the canonical deployment of the gas token. */
export function getCanonicalGasToken(): ProtocolContract {
  const contract = getCanonicalProtocolContract(GasTokenArtifact, 1);
  if (!contract.address.equals(GasTokenAddress)) {
    throw new Error(
      `Incorrect address for gas token (got ${contract.address.toString()} but expected ${GasTokenAddress.toString()}).`,
    );
  }
  return contract;
}

export const GasTokenAddress = AztecAddress.fromBigInt(GAS_TOKEN_ADDRESS);

export { GasTokenArtifact };
