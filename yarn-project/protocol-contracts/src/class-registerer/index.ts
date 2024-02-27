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
  '0x251ab40f36a148a5ac88fe3c2398ec1968d5676b0066299278610b870d738a9e',
);
