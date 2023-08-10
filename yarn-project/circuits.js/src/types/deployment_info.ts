import { AztecAddress, Fr, PartialAddress, PublicKey } from '../index.js';

/**
 * Represents the data generated as part of contract deployment.
 */
export type DeploymentInfo = {
  /**
   * The derived aztec address of the contract.
   */
  address: AztecAddress;
  /**
   * The partially derived aztec address of the contract.
   */
  partialAddress: PartialAddress;
  /**
   * The contract's constructor hash.
   */
  constructorHash: Fr;
  /**
   * The root of the contract's function tree.
   */
  functionTreeRoot: Fr;
  /**
   * The public key associated with the contract.
   */
  publicKey: PublicKey;
};
