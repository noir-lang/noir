import { AztecAddress, REGISTERER_CONTRACT_ADDRESS } from '@aztec/circuits.js';

import { type ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { ContractClassRegistererArtifact } from './artifact.js';

/** Returns the canonical deployment of the class registerer contract. */
export function getCanonicalClassRegisterer(): ProtocolContract {
  const contract = getCanonicalProtocolContract(ContractClassRegistererArtifact, 1);
  if (!contract.address.equals(ClassRegistererAddress)) {
    throw new Error(
      `Incorrect address for class registerer (got ${contract.address.toString()} but expected ${ClassRegistererAddress.toString()}).`,
    );
  }
  return contract;
}

export const ClassRegistererAddress = AztecAddress.fromBigInt(REGISTERER_CONTRACT_ADDRESS);
