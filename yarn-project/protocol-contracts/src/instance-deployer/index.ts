import { AztecAddress, DEPLOYER_CONTRACT_ADDRESS } from '@aztec/circuits.js';

import { type ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { ContractInstanceDeployerArtifact } from './artifact.js';

/** Returns the canonical deployment of the instance deployer contract. */
export function getCanonicalInstanceDeployer(): ProtocolContract {
  return getCanonicalProtocolContract(ContractInstanceDeployerArtifact, 1);
}

export const InstanceDeployerAddress = AztecAddress.fromBigInt(DEPLOYER_CONTRACT_ADDRESS);
