import { type FunctionCall } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/circuits.js';

import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Does not pay fees. Will work until we enforce fee payment for all txs.
 */
export class NoFeePaymentMethod implements FeePaymentMethod {
  constructor() {}

  getAsset() {
    return AztecAddress.ZERO;
  }

  getFunctionCalls(): Promise<FunctionCall[]> {
    return Promise.resolve([]);
  }

  getFeePayer(): Promise<AztecAddress> {
    return Promise.resolve(AztecAddress.ZERO);
  }
}
