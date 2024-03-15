import { AztecAddress } from '@aztec/circuits.js';

import { ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { ContractClassRegistererArtifact } from './artifact.js';

/** Returns the canonical deployment of the class registerer contract. */
export function getCanonicalClassRegisterer(): ProtocolContract {
  return getCanonicalProtocolContract(ContractClassRegistererArtifact, 1);
}

/**
 * Address of the canonical class registerer.
 * @remarks This should not change often, hence we hardcode it to save from having to recompute it every time.
 */
export const ClassRegistererAddress = AztecAddress.fromString(
  '0x2140db629d95644ef26140fa5ae87749ae28d373176af9a2e458052ced96c7b3',
);
