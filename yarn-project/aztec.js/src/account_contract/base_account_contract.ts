import { ContractArtifact } from '@aztec/foundation/abi';
import { CompleteAddress, NodeInfo } from '@aztec/types';

import { DefaultAccountInterface } from '../account/defaults/default_interface.js';
import { AccountInterface, AuthWitnessProvider } from '../account/interface.js';
import { AccountContract } from './account_contract.js';

/**
 * Base class for implementing an account contract. Requires that the account uses the
 * default entrypoint method signature.
 */
export abstract class BaseAccountContract implements AccountContract {
  abstract getAuthWitnessProvider(address: CompleteAddress): AuthWitnessProvider;
  abstract getDeploymentArgs(): any[];

  constructor(private artifact: ContractArtifact) {}

  getContractArtifact(): ContractArtifact {
    return this.artifact;
  }

  getInterface(address: CompleteAddress, nodeInfo: NodeInfo): AccountInterface {
    return new DefaultAccountInterface(this.getAuthWitnessProvider(address), address, nodeInfo);
  }
}
