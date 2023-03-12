import { EthAddress } from '../../eth_address/index.js';
import {
  bigIntToHex,
  bufferToHex,
  hexToBigInt,
  hexToBuffer,
  hexToNumber,
  numberToHex,
} from '../../hex_string/index.js';
import {
  fromRawTransactionResponse,
  RawTransactionResponse,
  toRawTransactionResponse,
  TransactionResponse,
} from './transaction_response.js';

export interface RawBlockHeaderResponse {
  hash: string | null;
  parentHash: string;
  sha3Uncles: string;
  miner: string;
  stateRoot: string;
  transactionsRoot: string;
  receiptsRoot: string;
  logsBloom: string | null;
  difficulty: string;
  number: string | null;
  gasLimit: string;
  gasUsed: string;
  timestamp: string;
  extraData: string;
  nonce: string | null;
  baseFeePerGas: string | null;
}

export interface RawBlockResponse extends RawBlockHeaderResponse {
  totalDifficulty: string;
  size: string;
  transactions: (RawTransactionResponse | string)[];
  uncles: string[];
}

export interface BlockHeaderResponse {
  hash: Buffer | null;
  parentHash: Buffer;
  sha3Uncles: Buffer;
  miner: EthAddress;
  stateRoot: Buffer;
  transactionsRoot: Buffer;
  receiptsRoot: Buffer;
  logsBloom: Buffer | null;
  difficulty: bigint;
  number: number | null;
  gasLimit: number;
  gasUsed: number;
  timestamp: number;
  extraData: Buffer;
  nonce: Buffer | null;
  baseFeePerGas: bigint | null;
}

export interface BlockResponse<T = TransactionResponse | Buffer> extends BlockHeaderResponse {
  totalDifficulty: bigint;
  size: number;
  transactions: T[];
  uncles: string[];
}

export function toRawBlockHeaderResponse(block: BlockHeaderResponse): RawBlockHeaderResponse {
  return {
    hash: block.hash ? bufferToHex(block.hash) : null,
    parentHash: bufferToHex(block.parentHash),
    sha3Uncles: bufferToHex(block.sha3Uncles),
    miner: block.miner.toString(),
    stateRoot: bufferToHex(block.stateRoot),
    transactionsRoot: bufferToHex(block.transactionsRoot),
    receiptsRoot: bufferToHex(block.receiptsRoot),
    logsBloom: block.logsBloom ? bufferToHex(block.logsBloom) : null,
    difficulty: bigIntToHex(block.difficulty),
    number: block.number !== null ? numberToHex(block.number)! : null,
    gasLimit: numberToHex(block.gasLimit)!,
    gasUsed: numberToHex(block.gasUsed)!,
    timestamp: numberToHex(block.timestamp)!,
    extraData: bufferToHex(block.extraData),
    nonce: block.nonce !== null ? bufferToHex(block.nonce) : null,
    baseFeePerGas: block.baseFeePerGas !== null ? bigIntToHex(block.baseFeePerGas) : null,
  };
}

export function toRawBlockResponse(block: BlockResponse): RawBlockResponse {
  return {
    ...toRawBlockHeaderResponse(block),
    totalDifficulty: bigIntToHex(block.totalDifficulty),
    size: numberToHex(block.size)!,
    transactions: block.transactions.map(tx => (Buffer.isBuffer(tx) ? bufferToHex(tx) : toRawTransactionResponse(tx))),
    uncles: block.uncles,
  };
}

export function fromRawBlockHeaderResponse(block: RawBlockHeaderResponse): BlockHeaderResponse {
  return {
    hash: block.hash ? hexToBuffer(block.hash) : null,
    parentHash: hexToBuffer(block.parentHash),
    sha3Uncles: hexToBuffer(block.sha3Uncles),
    miner: EthAddress.fromString(block.miner),
    stateRoot: hexToBuffer(block.stateRoot),
    transactionsRoot: hexToBuffer(block.transactionsRoot),
    receiptsRoot: hexToBuffer(block.receiptsRoot),
    logsBloom: block.logsBloom ? hexToBuffer(block.logsBloom) : null,
    difficulty: hexToBigInt(block.difficulty),
    number: block.number ? hexToNumber(block.number) : null,
    gasLimit: hexToNumber(block.gasLimit),
    gasUsed: hexToNumber(block.gasUsed),
    timestamp: hexToNumber(block.timestamp),
    extraData: hexToBuffer(block.extraData),
    nonce: block.nonce ? hexToBuffer(block.nonce) : null,
    baseFeePerGas: block.baseFeePerGas ? hexToBigInt(block.baseFeePerGas) : null,
  };
}

export function fromRawBlockResponse(block: RawBlockResponse): BlockResponse {
  return {
    ...fromRawBlockHeaderResponse(block),
    totalDifficulty: hexToBigInt(block.totalDifficulty),
    size: hexToNumber(block.size),
    transactions: block.transactions.map(tx =>
      typeof tx === 'string' ? hexToBuffer(tx) : fromRawTransactionResponse(tx),
    ),
    uncles: block.uncles,
  };
}
