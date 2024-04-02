import { type AztecAddress, type EthAddress, Point } from '@aztec/circuits.js';

import { type ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { GasTokenArtifact } from './artifact.js';

/** Returns the canonical deployment of the gas token. */
export function getCanonicalGasToken(l1Bridge: EthAddress): ProtocolContract {
  return getCanonicalProtocolContract(GasTokenArtifact, 1, [], Point.ZERO, l1Bridge);
}

export function getCanonicalGasTokenAddress(l1Bridge: EthAddress): AztecAddress {
  return getCanonicalGasToken(l1Bridge).address;
}
