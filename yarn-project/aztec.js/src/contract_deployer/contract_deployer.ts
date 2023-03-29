import { AztecRPCClient, ContractAbi } from '@aztec/aztec-rpc';
import { ConstructorMethod } from './constructor_method.js';

/**
 * A class for deploying contract.
 */
export class ContractDeployer {
  constructor(private abi: ContractAbi, private arc: AztecRPCClient) {}

  public deploy(...args: any[]) {
    return new ConstructorMethod(this.arc, this.abi, args);
  }
}
