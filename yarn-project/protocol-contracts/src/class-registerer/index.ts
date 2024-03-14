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
  '0x1ff58462bc433d2e277ca6371c796fcc2dbfae869edf87b60f7fdff335ae8baa',
);
