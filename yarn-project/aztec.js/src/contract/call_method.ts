import { AztecAddress, AztecRPCClient } from '@aztec/aztec-rpc';
import { ContractFunction } from './contract_function.js';

export interface CallMethodOptions {
  from?: AztecAddress;
}

/**
 * This is the class that is returned when calling e.g. `contract.methods.myMethod(arg0, arg1)`.
 * It contains available interactions one can call on a `send` method.
 */
export class CallMethod {
  constructor(
    private arc: AztecRPCClient,
    private contractAddress: AztecAddress,
    private entry: ContractFunction,
    private args: any[],
    private defaultOptions: CallMethodOptions = {},
  ) {}

  public call(options: CallMethodOptions = {}) {
    return Promise.resolve();
  }
}
