import { AztecAddress, FunctionData } from '@aztec/circuits.js';
import { FunctionAbi, FunctionType, encodeArguments } from '@aztec/foundation/abi';
import { FunctionCall, TxExecutionRequest } from '@aztec/types';

import { Wallet } from '../aztec_rpc_client/wallet.js';
import { BaseContractInteraction, SendMethodOptions } from './base_contract_interaction.js';

export { SendMethodOptions };

/**
 * Represents the options for a view method in a contract function interaction.
 * Allows specifying the address from which the view method should be called.
 */
export interface ViewMethodOptions {
  /**
   * The sender's Aztec address.
   */
  from?: AztecAddress;
}

/**
 * This is the class that is returned when calling e.g. `contract.methods.myMethod(arg0, arg1)`.
 * It contains available interactions one can call on a method, including view.
 */
export class ContractFunctionInteraction extends BaseContractInteraction {
  constructor(
    protected wallet: Wallet,
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
   * @returns A Promise that resolves to a transaction instance.
   */
  public async create(): Promise<TxExecutionRequest> {
    if (this.functionDao.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `create` on an unconstrained function.");
    }
    if (!this.txRequest) {
      this.txRequest = await this.wallet.createTxExecutionRequest([this.request()]);
    }
    return this.txRequest;
  }

  /**
   * Returns an execution request that represents this operation. Useful as a building
   * block for constructing batch requests.
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns An execution request wrapped in promise.
   */
  public request(): FunctionCall {
    const args = encodeArguments(this.functionDao, this.args);
    const functionData = FunctionData.fromAbi(this.functionDao);
    return { args, functionData, to: this.contractAddress };
  }

  /**
   * Execute a view (read-only) transaction on an unconstrained function.
   * This method is used to call functions that do not modify the contract state and only return data.
   * Throws an error if called on a non-unconstrained function.
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns The result of the view transaction as returned by the contract function.
   */
  public view(options: ViewMethodOptions = {}) {
    if (this.functionDao.functionType !== FunctionType.UNCONSTRAINED) {
      throw new Error('Can only call `view` on an unconstrained function.');
    }

    const { from } = options;
    return this.wallet.viewTx(this.functionDao.name, this.args, this.contractAddress, from);
  }
}
