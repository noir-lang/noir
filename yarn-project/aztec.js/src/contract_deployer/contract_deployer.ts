import { AztecRPCClient, ContractAbi } from '@aztec/aztec-rpc';
import { DeployMethod } from './deploy_method.js';

/**
 * A class for deploying contract.
 */
export class ContractDeployer {
  constructor(private abi: ContractAbi, private arc: AztecRPCClient) {}

  public deploy(...args: any[]) {
    return new DeployMethod(this.arc, this.abi, args);
  }
}
