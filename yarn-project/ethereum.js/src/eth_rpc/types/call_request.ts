import { EthAddress } from '../../eth_address/index.js';
import {
  bigIntToHex,
  bufferToHex,
  hexToBigInt,
  hexToBuffer,
  hexToNumber,
  numberToHex,
} from '../../hex_string/index.js';

export interface CallRequest {
  from?: EthAddress;
  to?: EthAddress;
  gas?: number;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  value?: bigint;
  data?: Buffer;
}

export interface RawCallRequest {
  from?: string;
  to?: string;
  gas?: string;
  maxFeePerGas?: string;
  maxPriorityFeePerGas?: string;
  value?: string;
  data?: string;
}

export function toRawCallRequest(tx: CallRequest): RawCallRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, data } = tx;
  return {
    from: from ? from.toString().toLowerCase() : undefined,
    to: to ? to.toString().toLowerCase() : undefined,
    data: data ? bufferToHex(data) : undefined,
    value: value ? bigIntToHex(value) : undefined,
    gas: gas ? numberToHex(gas) : undefined,
    maxFeePerGas: maxFeePerGas ? bigIntToHex(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? bigIntToHex(maxPriorityFeePerGas) : undefined,
  };
}

export function fromRawCallRequest(tx: RawCallRequest): CallRequest {
  const { from, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, data } = tx;
  return {
    from: from ? EthAddress.fromString(from) : undefined,
    to: to ? EthAddress.fromString(to) : undefined,
    data: data ? hexToBuffer(data) : undefined,
    value: value ? hexToBigInt(value) : undefined,
    gas: gas ? hexToNumber(gas) : undefined,
    maxFeePerGas: maxFeePerGas ? hexToBigInt(maxFeePerGas) : undefined,
    maxPriorityFeePerGas: maxPriorityFeePerGas ? hexToBigInt(maxPriorityFeePerGas) : undefined,
  };
}
