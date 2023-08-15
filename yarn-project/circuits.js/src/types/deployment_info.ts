import { CompleteAddress, Fr } from '../index.js';

/**
 * Represents the data generated as part of contract deployment.
 */
export type DeploymentInfo = {
  /**
   * The complete address of the deployed contract.
   */
  completeAddress: CompleteAddress;
  /**
   * The contract's constructor hash.
   */
  constructorHash: Fr;
  /**
   * The root of the contract's function tree.
   */
  functionTreeRoot: Fr;
};
