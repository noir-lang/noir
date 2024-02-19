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
  '0x1ae57857d2d234762b4a660986d94dfa361b06a3094d7c77dd0d133078516aa8',
);
