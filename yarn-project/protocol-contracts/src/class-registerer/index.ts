import { type AztecAddress } from '@aztec/circuits.js';

import { type ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { ContractClassRegistererArtifact } from './artifact.js';

/** Returns the canonical deployment of the class registerer contract. */
export function getCanonicalClassRegisterer(): ProtocolContract {
  return getCanonicalProtocolContract(ContractClassRegistererArtifact, 1);
}

let address: AztecAddress | undefined = undefined;

/** Returns the address for the canonical deployment of the class registerer */
export function getCanonicalClassRegistererAddress() {
  if (!address) {
    address = getCanonicalClassRegisterer().address;
  }
  return address;
}
