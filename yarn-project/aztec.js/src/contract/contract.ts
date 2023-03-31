import { AztecAddress, EthAddress } from '@aztec/foundation';
import { AztecRPCClient, DeployedContract } from '@aztec/aztec-rpc';
import { ContractAbi, FunctionAbi } from '@aztec/noir-contracts';
import { ContractFunctionInteraction } from './contract_function_interaction.js';

export class Contract {
  public methods: { [name: string]: (...args: any[]) => ContractFunctionInteraction } = {};

  constructor(public readonly address: AztecAddress, public readonly abi: ContractAbi, private arc: AztecRPCClient) {
    abi.functions.forEach((f: FunctionAbi) => {
      this.methods[f.name] = (...args: any[]) => {
        return new ContractFunctionInteraction(this.arc, this.address!, f.name, args, f.functionType);
      };
    });
  }

  attach(portalContract: EthAddress, dependencies: DeployedContract[] = []) {
    const deployedContract: DeployedContract = {
      abi: this.abi,
      address: this.address,
      portalContract,
    };
    return this.arc.addContracts([deployedContract, ...dependencies]);
  }
}
