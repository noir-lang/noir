import { EthAddress } from '@aztec/foundation';
import {
  bigIntToHex,
  bufferToHex,
  hexToBigInt,
  hexToBuffer,
  hexToNumber,
  numberToHex,
} from '../../hex_string/index.js';

export interface EstimateRequest {
  from?: EthAddress;
  to?: EthAddress;
  gas?: number;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  value?: bigint;
  data?: Buffer;
}

export interface RawEstimateRequest {
  from?: string;
  to?: string;
  gas?: string;
  maxFeePerGas?: string;
  maxPriorityFeePerGas?: string;
  value?: string;
  data?: string;
}

export function toRawEstimateRequest(tx: EstimateRequest): RawEstimateRequest {
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

export function fromRawEstimateRequest(tx: RawEstimateRequest): EstimateRequest {
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
