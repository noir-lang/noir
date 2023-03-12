import { EthAddress } from '../../eth_address/index.js';
import {
  bigIntToHex,
  bufferToHex,
  hexToBigInt,
  hexToBuffer,
  hexToNumber,
  numberToHex,
} from '../../hex_string/index.js';

export interface RawTransactionResponse {
  blockHash: string | null;
  blockNumber: string | null;
  from: string;
  gas: string;
  gasPrice: string;
  maxFeePerGas?: string;
  maxPriorityFeePerGas?: string;
  hash: string;
  input: string;
  nonce: string;
  to: string | null;
  transactionIndex: string | null;
  type: string;
  value: string;
  v: string;
  r: string;
  s: string;
}

export interface TransactionResponse {
  blockHash: string | null;
  blockNumber: number | null;
  from: EthAddress;
  gas: number;
  gasPrice: bigint;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  hash: string;
  input: Buffer;
  nonce: number;
  to: EthAddress | null;
  transactionIndex: number | null;
  type: number;
  value: bigint;
  v: string;
  r: string;
  s: string;
}

export function fromRawTransactionResponse(tx: RawTransactionResponse): TransactionResponse {
  return {
    ...tx,
    blockNumber: tx.blockNumber ? hexToNumber(tx.blockNumber) : null,
    transactionIndex: tx.transactionIndex ? hexToNumber(tx.transactionIndex) : null,
    nonce: hexToNumber(tx.nonce),
    gas: hexToNumber(tx.gas),
    gasPrice: hexToBigInt(tx.gasPrice),
    maxFeePerGas: tx.maxFeePerGas ? hexToBigInt(tx.gasPrice) : undefined,
    maxPriorityFeePerGas: tx.maxPriorityFeePerGas ? hexToBigInt(tx.gasPrice) : undefined,
    value: hexToBigInt(tx.value),
    type: hexToNumber(tx.type),
    to: tx.to ? EthAddress.fromString(tx.to) : null,
    from: EthAddress.fromString(tx.from),
    input: hexToBuffer(tx.input),
  };
}

export function toRawTransactionResponse(tx: TransactionResponse): RawTransactionResponse {
  return {
    ...tx,
    blockNumber: tx.blockNumber ? numberToHex(tx.blockNumber) : null,
    transactionIndex: tx.transactionIndex ? numberToHex(tx.transactionIndex) : null,
    nonce: numberToHex(tx.nonce)!,
    gas: numberToHex(tx.gas)!,
    gasPrice: bigIntToHex(tx.gasPrice),
    maxFeePerGas: tx.maxFeePerGas ? bigIntToHex(tx.maxFeePerGas) : undefined,
    maxPriorityFeePerGas: tx.maxPriorityFeePerGas ? bigIntToHex(tx.maxPriorityFeePerGas) : undefined,
    value: bigIntToHex(tx.value),
    type: numberToHex(tx.type),
    to: tx.to ? tx.to.toString() : null,
    from: tx.from.toString(),
    input: bufferToHex(tx.input),
  };
}
