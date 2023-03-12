import { EthAddress } from '../eth_address/index.js';
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

export interface Options {
  from?: EthAddress;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  gas?: number;
}

export interface CallOptions extends Options {
  value?: bigint;
}

export interface SendOptions extends CallOptions {
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
 * This is the class that is returned when calling e.g. `contract.methods.myMethod(arg1, arg2)`
 * It represents an interaction that can occur with that method and arguments. Interactions are:
 * - `estimateGas`
 * - `call`
 * - `send`
 * - `encodeAbi`
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

  public async estimateGas(options: CallOptions = {}) {
    try {
      return await this.eth.estimateGas(this.getCallRequest(options));
    } catch (err: any) {
      this.handleError(err);
    }
  }

  public async call(options: CallOptions = {}, block?: NumberOrTag) {
    try {
      const result = await this.eth.call(this.getCallRequest(options), block);
      return this.contractEntry.decodeReturnValue(result);
    } catch (err: any) {
      this.handleError(err);
    }
  }

  public send(options: SendOptions): SentTx {
    const tx = this.getTxRequest(options);

    if (!this.contractEntry.payable && tx.value !== undefined && tx.value > 0) {
      throw new Error('Cannot send value to non-payable contract method.');
    }

    const promise = this.eth.sendTransaction(tx).getTxHash();

    return new SentContractTx(this.eth, this.contractAbi, promise);
  }

  public encodeABI() {
    return this.contractEntry.encodeABI(this.args);
  }

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

  private getCallRequest(options: CallOptions = {}): CallRequest {
    return {
      ...this.defaultOptions,
      ...options,
      to: this.contractAddress!,
      data: this.encodeABI(),
    };
  }

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
