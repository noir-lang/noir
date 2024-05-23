import { type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { GasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Pay fee directly in the native gas token.
 */
export class NativeFeePaymentMethod implements FeePaymentMethod {
  constructor(protected sender: AztecAddress) {}

  getAsset() {
    return GasTokenAddress;
  }

  getFunctionCalls(): Promise<FunctionCall[]> {
    return Promise.resolve([]);
  }

  getFeePayer(): Promise<AztecAddress> {
    return Promise.resolve(this.sender);
  }
}
