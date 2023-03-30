import { AztecAddress, AztecRPCClient } from '@aztec/aztec-rpc';
import { ContractAbi, FunctionAbi } from '@aztec/noir-contracts';
import { ContractFunctionInteraction } from './contract_function_interaction.js';

export class Contract {
  public methods: { [name: string]: (...args: any[]) => ContractFunctionInteraction } = {};
  private address?: AztecAddress;

  constructor(public readonly abi: ContractAbi, private arc: AztecRPCClient) {
    abi.functions.forEach((f: FunctionAbi) => {
      this.methods[f.name] = (...args: any[]) => {
        return new ContractFunctionInteraction(this.arc, this.address!, f.name, args, f.functionType);
      };
    });
  }

  /*
  TODO implement a function to add the contract in the ARS
  */
  attach(address: AztecAddress) {
    this.address = address;
    return Promise.resolve();
  }
}
