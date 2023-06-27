import { AztecRPC, DeployedContract, generateFunctionSelector } from '@aztec/aztec-rpc';
import { ContractAbi, FunctionAbi } from '@aztec/foundation/abi';
import { ContractFunctionInteraction } from './contract_function_interaction.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

/**
 * Type representing a contract method that returns a ContractFunctionInteraction instance
 * and has a readonly 'selector' property of type Buffer. Takes any number of arguments.
 */
type ContractMethod = ((...args: any[]) => ContractFunctionInteraction) & {
  /**
   * The unique identifier for a contract function in bytecode.
   */
  readonly selector: Buffer;
};

/**
 * The Contract class represents a contract and provides utility methods for interacting with it.
 * It enables the creation of ContractFunctionInteraction instances for each function in the contract's ABI,
 * allowing users to call or send transactions to these functions. Additionally, the Contract class can be used
 * to attach the contract instance to a deployed contract on-chain through the AztecRPCClient, which facilitates
 * interaction with Aztec's privacy protocol.
 */
export class Contract {
  /**
   * An object containing contract methods mapped to their respective names.
   */
  public methods: { [name: string]: ContractMethod } = {};

  constructor(
    /**
     * The deployed contract's address.
     */
    public readonly address: AztecAddress,
    /**
     * The Application Binary Interface for the contract.
     */
    public readonly abi: ContractAbi,
    private arc: AztecRPC,
  ) {
    abi.functions.forEach((f: FunctionAbi) => {
      const interactionFunction = (...args: any[]) => {
        return new ContractFunctionInteraction(this.arc, this.address!, f.name, args, f.functionType);
      };
      this.methods[f.name] = Object.assign(interactionFunction, {
        /**
         * A getter for users to fetch the function selector.
         *
         * @returns Selector of the function.
         */
        get selector() {
          return generateFunctionSelector(f.name, f.parameters);
        },
      });
    });
  }

  /**
   * Attach the current contract instance to a portal contract and optionally add its dependencies.
   * The function will return a promise that resolves when all contracts have been added to the AztecRPCClient.
   * This is useful when you need to interact with a deployed contract that has multiple nested contracts.
   *
   * @param portalContract - The Ethereum address of the portal contract.
   * @param dependencies - An optional array of additional DeployedContract instances to be attached.
   * @returns A promise that resolves when all contracts are successfully added to the AztecRPCClient.
   */
  attach(portalContract: EthAddress, dependencies: DeployedContract[] = []) {
    const deployedContract: DeployedContract = {
      abi: this.abi,
      address: this.address,
      portalContract,
    };
    return this.arc.addContracts([deployedContract, ...dependencies]);
  }
}
