import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress, NodeInfo } from '@aztec/types';

import { Entrypoint } from '../index.js';

export * from './ecdsa_account_contract.js';
export * from './schnorr_account_contract.js';
export * from './single_key_account_contract.js';

/**
 * An account contract instance. Knows its ABI, deployment arguments, and to create transaction execution
 * requests out of function calls through an entrypoint.
 */
export interface AccountContract {
  /** Returns the ABI of this account contract. */
  getContractAbi(): ContractAbi;
  /** Returns the deployment arguments for this instance. */
  getDeploymentArgs(): Promise<any[]>;
  /**
   * Creates an entrypoint for creating transaction execution requests for this account contract.
   * @param address - Complete address of the deployed account contract.
   * @param nodeInfo - Chain id and protocol version where the account contract is deployed.
   */
  getEntrypoint(address: CompleteAddress, nodeInfo: NodeInfo): Promise<Entrypoint>;
}
