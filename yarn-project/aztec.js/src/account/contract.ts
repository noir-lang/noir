import { type CompleteAddress } from '@aztec/circuit-types';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type NodeInfo } from '@aztec/types/interfaces';

import { type AccountInterface } from './interface.js';

// docs:start:account-contract-interface
/**
 * An account contract instance. Knows its artifact, deployment arguments, how to create
 * transaction execution requests out of function calls, and how to authorize actions.
 */
export interface AccountContract {
  /**
   * Returns the artifact of this account contract.
   */
  getContractArtifact(): ContractArtifact;

  /**
   * Returns the deployment arguments for this instance, or undefined if this contract does not require deployment.
   */
  getDeploymentArgs(): any[] | undefined;

  /**
   * Returns the account interface for this account contract given a deployment at the provided address.
   * The account interface is responsible for assembling tx requests given requested function calls, and
   * for creating signed auth witnesses given action identifiers (message hashes).
   * @param address - Address where this account contract is deployed.
   * @param nodeInfo - Info on the chain where it is deployed.
   * @returns An account interface instance for creating tx requests and authorizing actions.
   */
  getInterface(address: CompleteAddress, nodeInfo: NodeInfo): AccountInterface;
}
// docs:end:account-contract-interface
