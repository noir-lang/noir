import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress, NodeInfo } from '@aztec/types';

import { DefaultAccountInterface } from '../defaults/default_interface.js';
import { AccountInterface, AuthWitnessProvider } from '../interface.js';
import { AccountContract } from './index.js';

/**
 * Base class for implementing an account contract. Requires that the account uses the
 * default entrypoint method signature.
 */
export abstract class BaseAccountContract implements AccountContract {
  abstract getAuthWitnessProvider(address: CompleteAddress): AuthWitnessProvider;
  abstract getDeploymentArgs(): Promise<any[]>;

  constructor(private abi: ContractAbi) {}

  getContractAbi(): ContractAbi {
    return this.abi;
  }

  getInterface(address: CompleteAddress, nodeInfo: NodeInfo): Promise<AccountInterface> {
    return Promise.resolve(new DefaultAccountInterface(this.getAuthWitnessProvider(address), address, nodeInfo));
  }
}
