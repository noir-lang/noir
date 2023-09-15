import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress, NodeInfo } from '@aztec/types';

import { AccountInterface } from '../interface.js';

export * from './ecdsa_account_contract.js';
export * from './schnorr_account_contract.js';
export * from './single_key_account_contract.js';
export * from './base_account_contract.js';

// docs:start:account-contract-interface
/**
 * An account contract instance. Knows its ABI, deployment arguments, how to create
 * transaction execution requests out of function calls, and how to authorize actions.
 */
export interface AccountContract {
  /**
   * Returns the ABI of this account contract.
   */
  getContractAbi(): ContractAbi;

  /**
   * Returns the deployment arguments for this instance.
   */
  getDeploymentArgs(): Promise<any[]>;

  /**
   * Returns the account interface for this account contract given a deployment at the provided address.
   * The account interface is responsible for assembling tx requests given requested function calls, and
   * for creating signed auth witnesses given action identifiers (message hashes).
   * @param address - Address where this account contract is deployed.
   * @param nodeInfo - Info on the chain where it is deployed.
   * @returns An account interface instance for creating tx requests and authorizing actions.
   */
  getInterface(address: CompleteAddress, nodeInfo: NodeInfo): Promise<AccountInterface>;
}
// docs:end:account-contract-interface
