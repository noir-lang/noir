import { EthAddress, Point } from '@aztec/circuits.js';

import { ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { GasTokenArtifact } from './artifact.js';

/** Returns the canonical deployment of the gas token. */
export function getCanonicalGasToken(): ProtocolContract {
  return getCanonicalProtocolContract(GasTokenArtifact, 1, [], Point.ZERO, EthAddress.ZERO);
}

export const GasTokenAddress = getCanonicalGasToken().address;
