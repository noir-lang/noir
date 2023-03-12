import { EthAddress } from '../eth_address/index.js';
import { EthSignature } from '../eth_sign/index.js';
import { TypedData } from '../eth_typed_data/index.js';
import { hexToBuffer } from '../hex_string/index.js';
import { EthereumProvider } from '../provider/index.js';
import { SentTx, SentTransaction } from './sent_tx.js';
import { TxHash } from './tx_hash.js';
import {
  CallRequest,
  fromRawBlockResponse,
  fromRawLogResponse,
  fromRawTransactionReceipt,
  fromRawTransactionResponse,
  NumberOrTag,
  numberOrTagToHex,
  RawLogResponse,
  toRawCallRequest,
  toRawEstimateRequest,
  toRawTransactionRequest,
  TransactionRequest,
} from './types/index.js';
import { LogRequest, toRawLogRequest } from './types/log_request.js';

/**
 * Provides a direct 1 to 1 mapping with the ethereum JSON-RPC specification.
 * https://ethereum.org/en/developers/docs/apis/json-rpc
 *
 * Types are marshalled to/from sensible types.
 * Number
 * BigInt
 * Buffer
 * TxHash
 * EthAddress
 */
export class EthereumRpc {
  constructor(private provider: EthereumProvider) {}

  public async protocolVersion() {
    const result = await this.provider.request({ method: 'eth_protocolVersion' });
    return Number(result);
  }

  public async syncing() {
    const result = await this.provider.request({ method: 'eth_syncing' });
    return {
      startingBlock: Number(result.startingBlock),
      currentBlock: Number(result.currentBlock),
      highestBlock: Number(result.highestBlock),
    };
  }

  public async getChainId() {
    const result = await this.provider.request({ method: 'eth_chainId' });
    return Number(result);
  }

  public async getCode(address: EthAddress, numberOrTag: NumberOrTag = 'latest') {
    const result = await this.provider.request({
      method: 'eth_getCode',
      params: [address.toString(), numberOrTagToHex(numberOrTag)],
    });
    return hexToBuffer(result);
  }

  public async getAccounts() {
    const result: string[] = await this.provider.request({
      method: 'eth_accounts',
    });
    return result.map(EthAddress.fromString);
  }

  public async getTransactionCount(addr: EthAddress) {
    const result = await this.provider.request({
      method: 'eth_getTransactionCount',
      params: [addr.toString(), 'latest'],
    });
    return Number(result);
  }

  public async getBalance(addr: EthAddress) {
    const result = await this.provider.request({
      method: 'eth_getBalance',
      params: [addr.toString(), 'latest'],
    });
    return BigInt(result);
  }

  public async getTransactionByHash(txHash: TxHash) {
    const result = await this.provider.request({
      method: 'eth_getTransactionByHash',
      params: [txHash.toString()],
    });
    return fromRawTransactionResponse(result);
  }

  public async getTransactionReceipt(txHash: TxHash) {
    const result = await this.provider.request({
      method: 'eth_getTransactionReceipt',
      params: [txHash.toString()],
    });
    return fromRawTransactionReceipt(result);
  }

  public async blockNumber() {
    const result = await this.provider.request({
      method: 'eth_blockNumber',
      params: [],
    });
    return Number(result);
  }

  public async getBlockByNumber(numberOrTag: NumberOrTag, fullTxs = false) {
    const result = await this.provider.request({
      method: 'eth_getBlockByNumber',
      params: [numberOrTagToHex(numberOrTag), fullTxs],
    });
    return result ? fromRawBlockResponse(result) : undefined;
  }

  public async sign(address: EthAddress, message: Buffer) {
    const result = await this.provider.request({
      method: 'eth_sign',
      params: [address.toString(), `0x${message.toString('hex')}`],
    });
    return EthSignature.fromString(result);
  }

  public async personalSign(message: Buffer, address: EthAddress) {
    const result = await this.provider.request({
      method: 'personal_sign',
      params: [`0x${message.toString('hex')}`, address.toString()],
    });
    return EthSignature.fromString(result);
  }

  public async signTypedDataV4(address: EthAddress, data: TypedData) {
    const result = await this.provider.request({
      method: 'eth_signTypedData_v4',
      params: [address.toString(), data],
    });
    return EthSignature.fromString(result);
  }

  public async estimateGas(tx: CallRequest) {
    const result = await this.provider.request({
      method: 'eth_estimateGas',
      params: [toRawEstimateRequest(tx)],
    });
    return Number(result);
  }

  public async call(tx: CallRequest, numberOrTag: NumberOrTag = 'latest') {
    const data = await this.provider.request({
      method: 'eth_call',
      params: [toRawCallRequest(tx), numberOrTagToHex(numberOrTag)],
    });
    return Buffer.from(data.slice(2), 'hex');
  }

  public sendTransaction(tx: TransactionRequest): SentTx {
    const txHashPromise = this.provider.request({
      method: 'eth_sendTransaction',
      params: [toRawTransactionRequest(tx)],
    });
    return new SentTransaction(this, txHashPromise);
  }

  public async signTransaction(tx: TransactionRequest) {
    const result = await this.provider.request({
      method: 'eth_signTransaction',
      params: [toRawTransactionRequest(tx)],
    });
    return hexToBuffer(result);
  }

  public async getLogs(logRequest: LogRequest) {
    const result: RawLogResponse[] = await this.provider.request({
      method: 'eth_getLogs',
      params: [toRawLogRequest(logRequest)],
    });
    return result.map(fromRawLogResponse);
  }
}
