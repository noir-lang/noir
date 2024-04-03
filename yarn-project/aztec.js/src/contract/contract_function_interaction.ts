import { type FunctionCall, PackedArguments, TxExecutionRequest } from '@aztec/circuit-types';
import { type AztecAddress, FunctionData, TxContext } from '@aztec/circuits.js';
import { type FunctionAbi, FunctionType, encodeArguments } from '@aztec/foundation/abi';

import { type Wallet } from '../account/wallet.js';
import { BaseContractInteraction, type SendMethodOptions } from './base_contract_interaction.js';

export { SendMethodOptions };

/**
 * Represents the options for simulating a contract function interaction.
 * Allows specifying the address from which the view method should be called.
 * Disregarded for simulation of public functions
 */
export type SimulateMethodOptions = {
  /**
   * The sender's Aztec address.
   */
  from?: AztecAddress;
};

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
   * @param opts - An optional object containing additional configuration for the transaction.
   * @returns A Promise that resolves to a transaction instance.
   */
  public async create(opts?: SendMethodOptions): Promise<TxExecutionRequest> {
    if (this.functionDao.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `create` on an unconstrained function.");
    }
    if (!this.txRequest) {
      this.txRequest = await this.wallet.createTxExecutionRequest([this.request()], opts?.fee);
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
    const functionData = FunctionData.fromAbi(this.functionDao);
    return { args, functionData, to: this.contractAddress };
  }

  /**
   * Simulate a transaction and get its return values
   * Differs from prove in a few important ways:
   * 1. It returns the values of the function execution
   * 2. It supports `unconstrained`, `private` and `public` functions
   * 3. For `private` execution it:
   * 3.a SKIPS the entrypoint and starts directly at the function
   * 3.b SKIPS public execution entirely
   * 4. For `public` execution it:
   * 4.a Removes the `txRequest` value after ended simulation
   * 4.b Ignores the `from` in the options
   *
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns The result of the transaction as returned by the contract function.
   */
  public async simulate(options: SimulateMethodOptions = {}): Promise<any> {
    if (this.functionDao.functionType == FunctionType.UNCONSTRAINED) {
      return this.wallet.viewTx(this.functionDao.name, this.args, this.contractAddress, options.from);
    }

    // TODO: If not unconstrained, we return a size 4 array of fields.
    // TODO: It should instead return the correctly decoded value
    // TODO: The return type here needs to be fixed! @LHerskind

    if (this.functionDao.functionType == FunctionType.SECRET) {
      const nodeInfo = await this.wallet.getNodeInfo();
      const packedArgs = PackedArguments.fromArgs(encodeArguments(this.functionDao, this.args));

      const txRequest = TxExecutionRequest.from({
        argsHash: packedArgs.hash,
        origin: this.contractAddress,
        functionData: FunctionData.fromAbi(this.functionDao),
        txContext: TxContext.empty(nodeInfo.chainId, nodeInfo.protocolVersion),
        packedArguments: [packedArgs],
        authWitnesses: [],
      });
      const simulatedTx = await this.pxe.simulateTx(txRequest, false, options.from ?? this.wallet.getAddress());
      return simulatedTx.privateReturnValues?.[0];
    } else {
      const txRequest = await this.create();
      const simulatedTx = await this.pxe.simulateTx(txRequest, true);
      this.txRequest = undefined;
      return simulatedTx.publicReturnValues?.[0];
    }
  }
}
