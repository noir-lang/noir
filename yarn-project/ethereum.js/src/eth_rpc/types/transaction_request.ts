import { EthAddress } from '@aztec/foundation';
import {
  bigIntToHex,
  bufferToHex,
  hexToBigInt,
  hexToBuffer,
  hexToNumber,
  numberToHex,
} from '../../hex_string/index.js';

export interface TransactionRequest {
  from: EthAddress;
  to?: EthAddress;
  gas?: number;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  value?: bigint;
  data?: Buffer;
  nonce?: number;
}

export type RawTransactionRequest = { [k in keyof TransactionRequest]: string };

export function toRawTransactionRequest(tx: TransactionRequest): RawTransactionRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, nonce, data } = tx;
  return {
    from: from.toString().toLowerCase(),
    to: to ? to.toString().toLowerCase() : undefined,
    gas: gas ? numberToHex(gas) : undefined,
    value: value ? bigIntToHex(value) : undefined,
    data: data ? bufferToHex(data) : undefined,
    nonce: nonce ? numberToHex(nonce) : undefined,
    maxFeePerGas: maxFeePerGas ? bigIntToHex(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? bigIntToHex(maxPriorityFeePerGas) : undefined,
  };
}

export function fromRawTransactionRequest(tx: RawTransactionRequest): TransactionRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, nonce, data } = tx;
  return {
    from: EthAddress.fromString(from),
    to: to ? EthAddress.fromString(to) : undefined,
    gas: gas ? hexToNumber(gas) : undefined,
    value: value ? hexToBigInt(value) : undefined,
    data: data ? hexToBuffer(data) : undefined,
    nonce: nonce ? hexToNumber(nonce) : undefined,
    maxFeePerGas: maxFeePerGas ? hexToBigInt(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? hexToBigInt(maxPriorityFeePerGas) : undefined,
  };
}
