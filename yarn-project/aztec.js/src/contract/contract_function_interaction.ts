import { AztecRPCClient, Tx, TxHash } from '@aztec/aztec-rpc';
import { AztecAddress, EcdsaSignature, Fr } from '@aztec/circuits.js';
import { FunctionType } from '@aztec/foundation/abi';
import { TxExecutionRequest } from '@aztec/types';
import { SentTx } from './sent_tx.js';

/**
 * Represents options for calling a (constrained) function in a contract.
 * Allows the user to specify the sender address and nonce for a transaction.
 */
export interface SendMethodOptions {
  /**
   * Sender's address initiating the transaction.
   */
  from?: AztecAddress;
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
  protected txRequest?: TxExecutionRequest;
  private signature?: EcdsaSignature;
  private tx?: Tx;

  constructor(
    protected arc: AztecRPCClient,
    protected contractAddress: AztecAddress,
    protected functionName: string,
    protected args: any[],
    protected functionType: FunctionType,
  ) {}

  /**
   * Send a request to create a transaction on a contract method with the specified options.
   * This function will generate a `TxRequest` object containing necessary information for
   * signing and executing the transaction. Throws an error if called on an unconstrained function.
   *
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns A TxRequest instance containing transaction details.
   */
  public async request(options: SendMethodOptions = {}) {
    if (this.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `request` on an unconstrained function.");
    }

    const { from } = options;
    this.txRequest = await this.arc.createTxRequest(this.functionName, this.args, this.contractAddress, from);
    return this.txRequest;
  }

  /**
   * Sign the transaction request for a contract method using the AztecRPCClient instance.
   * This function requires that `request` has been called before it, to generate a transaction request.
   * If not already generated, the transaction request is created by calling the `request` method.
   * Throws an error if the function type is unconstrained.
   *
   * @param options - An optional object containing additional configuration for the transaction.
   * @returns A Promise that resolves to an EcdsaSignature instance representing the signed transaction request.
   */
  public async sign(options: SendMethodOptions = {}) {
    if (this.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `sign` on an unconstrained function.");
    }

    if (!this.txRequest) {
      await this.request(options);
    }

    this.signature = await this.arc.signTxRequest(this.txRequest!);
    return this.signature;
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
  public async create(options: SendMethodOptions = {}) {
    if (this.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `create` on an unconstrained function.");
    }

    if (!this.signature) {
      await this.sign(options);
    }

    this.tx = await this.arc.createTx(this.txRequest!, this.signature!);
    return this.tx;
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
    if (this.functionType === FunctionType.UNCONSTRAINED) {
      throw new Error("Can't call `send` on an unconstrained function.");
    }

    let promise: Promise<TxHash>;
    if (this.tx) {
      promise = this.arc.sendTx(this.tx);
    } else {
      promise = (async () => {
        await this.create(options);
        return this.arc.sendTx(this.tx!);
      })();
    }

    return new SentTx(this.arc, promise);
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
    if (this.functionType !== FunctionType.UNCONSTRAINED) {
      throw new Error('Can only call `view` on an unconstrained function.');
    }

    const { from } = options;
    return this.arc.viewTx(this.functionName, this.args, this.contractAddress, from);
  }
}
