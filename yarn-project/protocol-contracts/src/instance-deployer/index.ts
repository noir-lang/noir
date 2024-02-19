import { AztecAddress } from '@aztec/circuits.js';

import { ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { ContractInstanceDeployerArtifact } from './artifact.js';

/** Returns the canonical deployment of the instance deployer contract. */
export function getCanonicalInstanceDeployer(): ProtocolContract {
  return getCanonicalProtocolContract(ContractInstanceDeployerArtifact, 1);
}

export const InstanceDeployerAddress = AztecAddress.fromString(
  '0x0747a20ed0c86035e44ea5606f30de459f40b55c5e82012640aa554546af9044',
);
