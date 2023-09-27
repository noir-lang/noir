import { Point } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { PXE, PublicKey } from '@aztec/types';

import { DeployMethod } from './deploy_method.js';

/**
 * A class for deploying contract.
 * @remarks Keeping this around even though we have Aztec.nr contract types because it can be useful for non-TS users.
 */
export class ContractDeployer {
  constructor(private abi: ContractAbi, private pxe: PXE, private publicKey?: PublicKey) {}

  /**
   * Deploy a contract using the provided ABI and constructor arguments.
   * This function creates a new DeployMethod instance that can be used to send deployment transactions
   * and query deployment status. The method accepts any number of constructor arguments, which will
   * be passed to the contract's constructor during deployment.
   *
   * @param args - The constructor arguments for the contract being deployed.
   * @returns A DeployMethod instance configured with the ABI, PXE, and constructor arguments.
   */
  public deploy(...args: any[]) {
    return new DeployMethod(this.publicKey ?? Point.ZERO, this.pxe, this.abi, args);
  }
}
