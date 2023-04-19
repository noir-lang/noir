import { EthAddress } from '@aztec/foundation';
import {
  CallRequest,
  EthereumRpc,
  NumberOrTag,
  SentTx,
  TransactionReceipt,
  TransactionRequest,
} from '../eth_rpc/index.js';
import { hexToBuffer } from '../hex_string/index.js';
import { ContractAbi, ContractFunctionEntry } from './abi/index.js';
import { decodeErrorFromContract } from './decode_error.js';
import { SentContractTx } from './sent_contract_tx.js';

/**
 * Represents the optional parameters for interacting with a contract.
 * Provides customization options like sender address, maximum gas fees, and gas limit.
 */
export interface Options {
  /**
   * The Ethereum address initiating the transaction.
   */
  from?: EthAddress;
  /**
   * The maximum fee per gas unit for the transaction.
   */
  maxFeePerGas?: bigint;
  /**
   * The maximum priority fee per gas unit for the transaction.
   */
  maxPriorityFeePerGas?: bigint;
  /**
   * The maximum amount of gas units to be used for the transaction.
   */
  gas?: number;
}

/**
 * Represents the call options for a contract function interaction.
 * These options include the sender address (from), maximum fee per gas (maxFeePerGas),
 * maximum priority fee per gas (maxPriorityFeePerGas), gas limit (gas), and value.
 */
export interface CallOptions extends Options {
  /**
   * The amount of ether (in wei) to transfer during the transaction.
   */
  value?: bigint;
}

/**
 * Represents the options for sending a transaction in the Ethereum network.
 * Provides optional parameters to control the execution of a transaction, such as gas limits, value, and nonce.
 */
export interface SendOptions extends CallOptions {
  /**
   * The nonce value representing the number of transactions sent from the sender's address.
   */
  nonce?: number;
}

/**
 * The interactions available when making a call.
 */
export interface TxCall<Return = any> {
  call(options?: CallOptions, block?: NumberOrTag): Promise<Return>;
  estimateGas(options?: CallOptions): Promise<number>;
  encodeABI(): Buffer;
}

/**
 * The interactions available when performing a tx send.
 */
export interface TxSend<TxReceipt = TransactionReceipt, Return = any> {
  call(options?: CallOptions, block?: NumberOrTag): Promise<Return>;
  send(options?: SendOptions): SentTx<TxReceipt>;
  estimateGas(options?: CallOptions): Promise<number>;
  encodeABI(): Buffer;
}

/**
 * This is the class that is returned when calling e.g. `contract.methods.myMethod(arg1, arg2)`.
 * It represents an interaction that can occur with that method and arguments. Interactions are:.
 * - `estimateGas`.
 * - `call`.
 * - `send`.
 * - `encodeAbi`.
 */
export class FunctionInteraction implements TxCall, TxSend {
  constructor(
    protected eth: EthereumRpc,
    protected contractEntry: ContractFunctionEntry,
    protected contractAbi: ContractAbi,
    protected contractAddress?: EthAddress,
    protected args: any[] = [],
    protected defaultOptions: Options = {},
  ) {}

  /**
   * Estimate the amount of gas required to perform a transaction for the function interaction.
   * The gas estimation is based on the provided 'options' object, which can include parameters such as 'from', 'maxFeePerGas', 'maxPriorityFeePerGas', and 'gas'.
   * If the transaction execution fails or there's an error in the call, it attempts to handle the error gracefully by providing a meaningful message.
   *
   * @param options - An optional object containing transaction parameters and overrides for the function interaction.
   * @returns A Promise that resolves to the estimated gas amount required for the transaction.
   * @throws Will throw an error if the call fails with a decoded error message, or a generic error message if decoding fails.
   */
  public async estimateGas(options: CallOptions = {}) {
    try {
      return await this.eth.estimateGas(this.getCallRequest(options));
    } catch (err: any) {
      this.handleError(err);
    }
  }

  /**
   * Executes a read-only contract function call, returning the decoded result.
   * This interaction does not require a transaction on the blockchain and is thus gas-free.
   * If the call encounters an error, it attempts to decode the error message from the contract
   * and throws an error with a meaningful message. Otherwise, it throws the original error.
   *
   * @param options - Optional settings specifying "from", "value", "maxFeePerGas", "maxPriorityFeePerGas" and "gas".
   * @param block - Optional specification of the block number or tag at which the call must be executed.
   * @returns The return value of the contract function call after successful decoding.
   */
  public async call(options: CallOptions = {}, block?: NumberOrTag) {
    try {
      const result = await this.eth.call(this.getCallRequest(options), block);
      return this.contractEntry.decodeReturnValue(result);
    } catch (err: any) {
      this.handleError(err);
    }
  }

  /**
   * Sends a transaction to the specified contract method with given options.
   * It returns a SentTx instance containing the transaction receipt and decoded return value (if any).
   * Throws an error if the from address is not specified or attempting to send value to a non-payable method.
   *
   * @param options - An object containing optional parameters: from, nonce, value, maxFeePerGas, maxPriorityFeePerGas, and gas.
   * @returns A SentTx instance representing the sent transaction.
   */
  public send(options: SendOptions): SentTx {
    const tx = this.getTxRequest(options);

    if (!this.contractEntry.payable && tx.value !== undefined && tx.value > 0) {
      throw new Error('Cannot send value to non-payable contract method.');
    }

    const promise = this.eth.sendTransaction(tx).getTxHash();

    return new SentContractTx(this.eth, this.contractAbi, promise);
  }

  /**
   * Encodes the ABI (Application Binary Interface) for the function interaction with the provided arguments.
   * The encoded ABI is a serialized representation of the function's signature and its arguments, which can be used
   * by the Ethereum client to process the method call or transaction. This is useful for encoding contract function
   * calls when interacting with the Ethereum blockchain.
   *
   * @returns A Buffer containing the encoded ABI for the function interaction.
   */
  public encodeABI() {
    return this.contractEntry.encodeABI(this.args);
  }

  /**
   * Construct a transaction request object by merging the provided send options with the default options, `from` address, contract address, and encoded ABI data.
   * This transaction request object is used for sending transactions to the Ethereum network.
   * Throws an error if the `from` address is not specified.
   *
   * @param options - The send options containing information required for constructing the transaction request object.
   * @returns A TransactionRequest instance with all necessary data for sending the transaction.
   */
  private getTxRequest(options: SendOptions = {}): TransactionRequest {
    const from = options.from || this.defaultOptions.from;
    if (!from) {
      throw new Error('You must specify a from address to send a tx.');
    }
    return {
      ...this.defaultOptions,
      ...options,
      from,
      to: this.contractAddress!,
      data: this.encodeABI(),
    };
  }

  /**
   * Constructs and returns a CallRequest object for the current contract function interaction.
   * The CallRequest object combines the provided options with the default options and includes
   * the encoded ABI data of the function call. This object can be used to perform various
   * interactions such as estimating gas, making calls, or sending transactions.
   *
   * @param options - An optional CallOptions object containing values such as from address,
   *                  maxFeePerGas, maxPriorityFeePerGas, gas, and value.
   * @returns A CallRequest object with the necessary information for further interactions.
   */
  private getCallRequest(options: CallOptions = {}): CallRequest {
    return {
      ...this.defaultOptions,
      ...options,
      to: this.contractAddress!,
      data: this.encodeABI(),
    };
  }

  /**
   * Handles errors occurring during the execution of a contract function call.
   * If the error data contains a decodable error message, throws an error with a decoded message.
   * Otherwise, throws the original error with its message.
   *
   * @param err - The error object caught during the contract function call execution.
   * @throws  An error with either the decoded error message or the original error message.
   */
  private handleError(err: any): never {
    if (err.data && err.data.length > 2) {
      const decoded = decodeErrorFromContract(this.contractAbi, hexToBuffer(err.data));
      if (decoded) {
        throw new Error(`call() failed: ${decoded.message}`);
      }
    }
    throw new Error(`call() failed: ${err.message}`);
  }
}
