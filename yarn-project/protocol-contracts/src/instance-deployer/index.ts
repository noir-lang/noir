import { AztecAddress } from '@aztec/circuits.js';

import { ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { ContractInstanceDeployerArtifact } from './artifact.js';

/** Returns the canonical deployment of the instance deployer contract. */
export function getCanonicalInstanceDeployer(): ProtocolContract {
  return getCanonicalProtocolContract(ContractInstanceDeployerArtifact, 1);
}

export const InstanceDeployerAddress = AztecAddress.fromString(
  '0x1799c61aa10430bf6fec46679c4cb76c3ed12cd8b6e73ed7389d5ae296ad1b97',
);
