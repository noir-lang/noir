import { type Fr } from '@aztec/foundation/fields';

import { type CompleteAddress } from '../structs/complete_address.js';

/**
 * Represents the data generated as part of contract deployment.
 */
export type DeploymentInfo = {
  /**
   * The complete address of the deployed contract.
   */
  completeAddress: CompleteAddress;
  /**
   * The contract's constructor verification key hash.
   */
  constructorVkHash: Fr;
  /**
   * The contract's constructor hash.
   */
  constructorHash: Fr;
  /**
   * The root of the contract's function tree.
   */
  functionTreeRoot: Fr;
};
