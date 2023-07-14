import { FunctionAbi } from '@aztec/foundation/abi';

import { AztecAddress, Fr, Point } from '../index.js';

/** Represents a user public key. */
export type PublicKey = Point;

/**
 * A type to represent a partially constructed contract address.
 */
export type PartialContractAddress = Fr;

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
  partialAddress: PartialContractAddress;
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

/**
 * A contract function Data Access Object (DAO).
 * Extends the FunctionAbi interface, adding a 'selector' property.
 * The 'selector' is a unique identifier for the function within the contract.
 */
export interface ContractFunctionDao extends FunctionAbi {
  /**
   * Unique identifier for a contract function.
   */
  selector: Buffer;
}
