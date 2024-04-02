import { type PublicKey } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { Point } from '@aztec/foundation/fields';

import { type Wallet } from '../account/wallet.js';
import { DeployMethod } from '../contract/deploy_method.js';
import { Contract } from '../contract/index.js';

/**
 * A class for deploying contract.
 * @remarks Keeping this around even though we have Aztec.nr contract types because it can be useful for non-TS users.
 */
export class ContractDeployer {
  constructor(
    private artifact: ContractArtifact,
    private wallet: Wallet,
    private publicKey?: PublicKey,
    private constructorName?: string,
  ) {}

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
    const postDeployCtor = (address: AztecAddress, wallet: Wallet) => Contract.at(address, this.artifact, wallet);
    return new DeployMethod(
      this.publicKey ?? Point.ZERO,
      this.wallet,
      this.artifact,
      postDeployCtor,
      args,
      this.constructorName,
    );
  }
}
