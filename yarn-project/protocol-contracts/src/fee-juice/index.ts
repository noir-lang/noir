import { AztecAddress, FEE_JUICE_ADDRESS } from '@aztec/circuits.js';

import { type ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { FeeJuiceArtifact } from './artifact.js';

/** Returns the canonical deployment of the Fee Juice. */
export function getCanonicalFeeJuice(): ProtocolContract {
  const contract = getCanonicalProtocolContract(FeeJuiceArtifact, 1);
  if (!contract.address.equals(FeeJuiceAddress)) {
    throw new Error(
      `Incorrect address for Fee Juice (got ${contract.address.toString()} but expected ${FeeJuiceAddress.toString()}).`,
    );
  }
  return contract;
}

export const FeeJuiceAddress = AztecAddress.fromBigInt(FEE_JUICE_ADDRESS);

export { FeeJuiceArtifact as FeeJuiceArtifact };
