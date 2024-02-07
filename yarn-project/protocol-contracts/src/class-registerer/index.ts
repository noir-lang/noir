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
  '0x29c0cd0000951bba8af520ad5513cc53d9f0413c5a24a72a4ba8c17894c0bef9',
);
