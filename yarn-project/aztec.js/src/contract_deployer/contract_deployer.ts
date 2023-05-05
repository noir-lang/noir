import { AztecRPCClient } from '@aztec/aztec-rpc';
import { ContractAbi } from '@aztec/noir-contracts';
import { DeployMethod } from './deploy_method.js';

/**
 * A class for deploying contract.
 */
export class ContractDeployer {
  constructor(private abi: ContractAbi, private arc: AztecRPCClient) {}

  /**
   * Deploy a contract using the provided ABI and constructor arguments.
   * This function creates a new DeployMethod instance that can be used to send deployment transactions
   * and query deployment status. The method accepts any number of constructor arguments, which will
   * be passed to the contract's constructor during deployment.
   *
   * @param args - The constructor arguments for the contract being deployed.
   * @returns A DeployMethod instance configured with the ABI, AztecRPCClient, and constructor arguments.
   */
  public deploy(...args: any[]) {
    return new DeployMethod(this.arc, this.abi, args);
  }
}
