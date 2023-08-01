import { AztecAddress, Fr, FunctionData, TxContext } from '@aztec/circuits.js';
import { FunctionAbi, FunctionType, encodeArguments } from '@aztec/foundation/abi';
import { ExecutionRequest, Tx, TxExecutionRequest } from '@aztec/types';

import { Wallet } from '../aztec_rpc_client/wallet.js';
import { SentTx } from './sent_tx.js';

/**
 * Represents options for calling a (constrained) function in a contract.
 * Allows the user to specify the sender address and nonce for a transaction.
 */
export interface SendMethodOptions {
  /**
   * Sender's address initiating the transaction.
   */
  origin?: AztecAddress;
  /**
   * The nonce representing the order of transactions sent by the address.
   */
  nonce?: Fr;
}

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
 * It contains available interactions one can call on a method.
 */
export class ContractFunctionInteraction {
  protected tx?: Tx;
  protected txRequest?: TxExecutionRequest;

  constructor(
    protected wallet: Wallet,
    protected contractAddress: AztecAddress,
    protected functionDao: FunctionAbi,
    protected args: any[],
  ) {
    if (args.some(arg => arg === undefined || arg === null)) {
      throw new Error('All function interaction arguments must be defined and not null. Received: ' + args);
    }
  }

  /**
   * Create an Aztec transaction instance by combining the transaction request and its signature.
   * This function will first check if a signature exists, and if not, it will call the `sign` method
   * to obtain the signature before creating the transaction. Throws an error if the function is
   * of unconstrained type or if the transaction request and signature are missing.
   *
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns A Promise that resolves to a transaction instance.
   */
  public async create(options: SendMethodOptions = {}): Promise<TxExecutionRequest> {
    if (this.functionDao.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `create` on an unconstrained function.");
    }
    if (!this.txRequest) {
      const executionRequest = this.getExecutionRequest(this.contractAddress, options.origin);
      const nodeInfo = await this.wallet.getNodeInfo();
      const txContext = TxContext.empty(new Fr(nodeInfo.chainId), new Fr(nodeInfo.version));
      const txRequest = await this.wallet.createAuthenticatedTxRequest([executionRequest], txContext);
      this.txRequest = txRequest;
    }
    return this.txRequest;
  }

  /**
   * Simulates a transaction's execution.
   * @param options - optional arguments to be used in the creation of the transaction
   * @returns The resulting transaction
   */
  public async simulate(options: SendMethodOptions = {}): Promise<Tx> {
    const txRequest = this.txRequest ?? (await this.create(options));
    this.tx = await this.wallet.simulateTx(txRequest);
    return this.tx;
  }

  /**
   * Returns an execution request that represents this operation. Useful as a building
   * block for constructing batch requests.
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns An execution request.
   */
  public request(options: SendMethodOptions = {}): ExecutionRequest {
    return this.getExecutionRequest(this.contractAddress, options.origin);
  }

  protected getExecutionRequest(to: AztecAddress, from?: AztecAddress): ExecutionRequest {
    const flatArgs = encodeArguments(this.functionDao, this.args);
    from = from ?? this.wallet.getAddress();

    return {
      args: flatArgs,
      functionData: FunctionData.fromAbi(this.functionDao),
      to,
      from,
    };
  }

  /**
   * Sends a transaction to the contract function with the specified options.
   * This function throws an error if called on an unconstrained function.
   * It creates and signs the transaction if necessary, and returns a SentTx instance,
   * which can be used to track the transaction status, receipt, and events.
   *
   * @param options - An optional object containing 'from' property representing
   * the AztecAddress of the sender. If not provided, the default address is used.
   * @returns A SentTx instance for tracking the transaction status and information.
   */
  public send(options: SendMethodOptions = {}) {
    if (this.functionDao.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `send` on an unconstrained function.");
    }

    const promise = (async () => {
      const tx = this.tx ?? (await this.simulate(options));
      return this.wallet.sendTx(tx);
    })();

    return new SentTx(this.wallet, promise);
  }

  /**
   * Execute a view (read-only) transaction on an unconstrained function.
   * This method is used to call functions that do not modify the contract state and only return data.
   * Throws an error if called on a non-unconstrained function.
   *
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
