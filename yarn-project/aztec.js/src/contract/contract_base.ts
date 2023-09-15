import { ContractAbi, FunctionAbi, FunctionSelector } from '@aztec/foundation/abi';
import { EthAddress } from '@aztec/foundation/eth-address';
import { CompleteAddress, DeployedContract } from '@aztec/types';

import { Wallet } from '../wallet/index.js';
import { ContractFunctionInteraction } from './contract_function_interaction.js';

/**
 * Type representing a contract method that returns a ContractFunctionInteraction instance
 * and has a readonly 'selector' property of type Buffer. Takes any number of arguments.
 */
export type ContractMethod = ((...args: any[]) => ContractFunctionInteraction) & {
  /**
   * The unique identifier for a contract function in bytecode.
   */
  readonly selector: FunctionSelector;
};

/**
 * Abstract implementation of a contract extended by the Contract class and generated contract types.
 */
export class ContractBase {
  /**
   * An object containing contract methods mapped to their respective names.
   */
  public methods: { [name: string]: ContractMethod } = {};

  protected constructor(
    /**
     * The deployed contract's complete address.
     */
    public readonly completeAddress: CompleteAddress,
    /**
     * The Application Binary Interface for the contract.
     */
    public readonly abi: ContractAbi,
    /**
     * The wallet.
     */
    protected wallet: Wallet,
  ) {
    abi.functions.forEach((f: FunctionAbi) => {
      const interactionFunction = (...args: any[]) => {
        return new ContractFunctionInteraction(this.wallet, this.completeAddress.address!, f, args);
      };

      this.methods[f.name] = Object.assign(interactionFunction, {
        /**
         * A getter for users to fetch the function selector.
         * @returns Selector of the function.
         */
        get selector() {
          return FunctionSelector.fromNameAndParameters(f.name, f.parameters);
        },
      });
    });
  }

  /**
   * Address of the contract.
   */
  public get address() {
    return this.completeAddress.address;
  }

  /**
   * Creates a new instance of the contract wrapper attached to a different wallet.
   * @param wallet - Wallet to use for sending txs.
   * @returns A new contract instance.
   */
  public withWallet(wallet: Wallet): this {
    return new ContractBase(this.completeAddress, this.abi, wallet) as this;
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
  public attach(portalContract: EthAddress, dependencies: DeployedContract[] = []) {
    const deployedContract: DeployedContract = {
      abi: this.abi,
      completeAddress: this.completeAddress,
      portalContract,
    };
    return this.wallet.addContracts([deployedContract, ...dependencies]);
  }
}
