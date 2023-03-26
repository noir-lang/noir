import { AztecRPCClient, ContractAbi } from '@aztec/aztec-rpc';
import { ConstructorMethod, ConstructorOptions } from './constructor_method.js';

/**
 * A class for deploying contract.
 */
export class ContractDeployer {
  constructor(private abi: ContractAbi, private arc: AztecRPCClient, private defaultOptions: ConstructorOptions = {}) {}

  public deploy(...args: any[]) {
    return new ConstructorMethod(this.arc, this.abi, args, this.defaultOptions);
  }
}
