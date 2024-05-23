import type { FunctionCall, TxExecutionRequest } from '@aztec/circuit-types';
import { type AztecAddress, type GasSettings } from '@aztec/circuits.js';
import {
  type FunctionAbi,
  FunctionSelector,
  FunctionType,
  decodeReturnValues,
  encodeArguments,
} from '@aztec/foundation/abi';

import { type Wallet } from '../account/wallet.js';
import { BaseContractInteraction, type SendMethodOptions } from './base_contract_interaction.js';

export { SendMethodOptions };

/**
 * Represents the options for simulating a contract function interaction.
 * Allows specifying the address from which the view method should be called.
 * Disregarded for simulation of public functions
 */
export type SimulateMethodOptions = {
  /** The sender's Aztec address. */
  from?: AztecAddress;
  /** Gas settings for the simulation. */
  gasSettings?: GasSettings;
};

/**
 * This is the class that is returned when calling e.g. `contract.methods.myMethod(arg0, arg1)`.
 * It contains available interactions one can call on a method, including view.
 */
export class ContractFunctionInteraction extends BaseContractInteraction {
  constructor(
    wallet: Wallet,
    protected contractAddress: AztecAddress,
    protected functionDao: FunctionAbi,
    protected args: any[],
  ) {
    super(wallet);
    if (args.some(arg => arg === undefined || arg === null)) {
      throw new Error('All function interaction arguments must be defined and not null. Received: ' + args);
    }
  }

  /**
   * Create a transaction execution request that represents this call, encoded and authenticated by the
   * user's wallet, ready to be simulated.
   * @param opts - An optional object containing additional configuration for the transaction.
   * @returns A Promise that resolves to a transaction instance.
   */
  public async create(opts?: SendMethodOptions): Promise<TxExecutionRequest> {
    if (this.functionDao.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `create` on an unconstrained function.");
    }
    if (!this.txRequest) {
      const calls = [this.request()];
      const fee = opts?.estimateGas ? await this.getFeeOptions({ calls, fee: opts?.fee }) : opts?.fee;
      this.txRequest = await this.wallet.createTxExecutionRequest({ calls, fee });
    }
    return this.txRequest;
  }

  /**
   * Returns an execution request that represents this operation. Useful as a building
   * block for constructing batch requests.
   * @returns An execution request wrapped in promise.
   */
  public request(): FunctionCall {
    const args = encodeArguments(this.functionDao, this.args);
    return {
      name: this.functionDao.name,
      args,
      selector: FunctionSelector.fromNameAndParameters(this.functionDao.name, this.functionDao.parameters),
      type: this.functionDao.functionType,
      to: this.contractAddress,
      isStatic: this.functionDao.isStatic,
      returnTypes: this.functionDao.returnTypes,
    };
  }

  /**
   * Simulate a transaction and get its return values
   * Differs from prove in a few important ways:
   * 1. It returns the values of the function execution
   * 2. It supports `unconstrained`, `private` and `public` functions
   *
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns The result of the transaction as returned by the contract function.
   */
  public async simulate(options: SimulateMethodOptions = {}): Promise<any> {
    if (this.functionDao.functionType == FunctionType.UNCONSTRAINED) {
      return this.wallet.simulateUnconstrained(this.functionDao.name, this.args, this.contractAddress, options?.from);
    }

    const txRequest = await this.create();
    const simulatedTx = await this.wallet.simulateTx(txRequest, true, options?.from);

    // As account entrypoints are private, for private functions we retrieve the return values from the first nested call
    // since we're interested in the first set of values AFTER the account entrypoint
    // For public functions we retrieve the first values directly from the public output.
    const rawReturnValues =
      this.functionDao.functionType == FunctionType.PRIVATE
        ? simulatedTx.privateReturnValues?.nested?.[0].values
        : simulatedTx.publicOutput?.publicReturnValues?.[0].values;

    return rawReturnValues ? decodeReturnValues(this.functionDao.returnTypes, rawReturnValues) : [];
  }
}
