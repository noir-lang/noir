import { FunctionAbi, generateFunctionSelector } from '@aztec/aztec-rpc';
import { encodeParameters } from '../abi_coder/index.js';

export class ContractFunction {
  constructor(private abi: FunctionAbi) {}

  public encodeABI() {
    return generateFunctionSelector(this.abi.name, this.abi.parameters);
  }

  public encodeParameters(args: any[]) {
    return encodeParameters(this.abi.parameters, args);
  }
}
