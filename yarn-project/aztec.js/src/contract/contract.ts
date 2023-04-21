import { AztecAddress, EthAddress } from '@aztec/foundation';
import { AztecRPCClient, DeployedContract, generateFunctionSelector } from '@aztec/aztec-rpc';
import { ContractAbi, FunctionAbi } from '@aztec/noir-contracts';
import { ContractFunctionInteraction } from './contract_function_interaction.js';

type ContractMethod = ((...args: any[]) => ContractFunctionInteraction) & { readonly selector: Buffer };

export class Contract {
  public methods: { [name: string]: ContractMethod } = {};

  constructor(public readonly address: AztecAddress, public readonly abi: ContractAbi, private arc: AztecRPCClient) {
    abi.functions.forEach((f: FunctionAbi) => {
      const interactionFunction = (...args: any[]) => {
        return new ContractFunctionInteraction(this.arc, this.address!, f.name, args, f.functionType);
      };
      this.methods[f.name] = Object.assign(interactionFunction, {
        get selector() {
          return generateFunctionSelector(f.name, f.parameters);
        },
      });
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
