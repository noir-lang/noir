import { EthAddress } from '@aztec/foundation';
import { fromRawLogResponse, LogResponse, RawLogResponse, toRawLogResponse } from './log_response.js';
import { numberToHex } from '../../hex_string/index.js';
import { TxHash } from '../tx_hash.js';

export interface RawTransactionReceipt {
  transactionHash: string;
  transactionIndex: string;
  blockHash: string;
  blockNumber: string;
  from: string;
  to: string | null;
  cumulativeGasUsed: string;
  gasUsed: string;
  contractAddress: string | null;
  logs: RawLogResponse[];
  status: string;
}

export interface TransactionReceipt {
  transactionHash: TxHash;
  transactionIndex: number;
  blockHash: string;
  blockNumber: number;
  from: EthAddress;
  to?: EthAddress;
  cumulativeGasUsed: number;
  gasUsed: number;
  contractAddress?: EthAddress;
  logs: LogResponse[];
  status: boolean;
}

export function fromRawTransactionReceipt(receipt?: RawTransactionReceipt): TransactionReceipt | undefined {
  if (!receipt || !receipt.blockHash) {
    return;
  }

  return {
    ...receipt,
    to: receipt.to ? EthAddress.fromString(receipt.to) : undefined,
    from: EthAddress.fromString(receipt.from),
    blockNumber: Number(receipt.blockNumber),
    transactionIndex: Number(receipt.transactionIndex),
    transactionHash: TxHash.fromString(receipt.transactionHash),
    cumulativeGasUsed: Number(receipt.cumulativeGasUsed),
    gasUsed: Number(receipt.gasUsed),
    logs: receipt.logs.map(fromRawLogResponse),
    contractAddress: receipt.contractAddress ? EthAddress.fromString(receipt.contractAddress) : undefined,
    status: Boolean(Number(receipt.status)),
  };
}

export function toRawTransactionReceipt(receipt: TransactionReceipt): RawTransactionReceipt {
  return {
    ...receipt,
    to: receipt.to ? receipt.to.toString() : null,
    from: receipt.from.toString(),
    blockNumber: numberToHex(receipt.blockNumber),
    transactionIndex: numberToHex(receipt.transactionIndex),
    transactionHash: receipt.transactionHash.toString(),
    cumulativeGasUsed: numberToHex(receipt.cumulativeGasUsed),
    gasUsed: numberToHex(receipt.gasUsed)!,
    logs: receipt.logs.map(toRawLogResponse),
    contractAddress: receipt.contractAddress ? receipt.contractAddress.toString() : null,
    status: numberToHex(receipt.status ? 1 : 0),
  };
}
