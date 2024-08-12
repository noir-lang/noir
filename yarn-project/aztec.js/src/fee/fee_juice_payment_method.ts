import { type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { FeeJuiceAddress } from '@aztec/protocol-contracts/fee-juice';

import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Pay fee directly in the Fee Juice.
 */
export class FeeJuicePaymentMethod implements FeePaymentMethod {
  constructor(protected sender: AztecAddress) {}

  getAsset() {
    return FeeJuiceAddress;
  }

  getFunctionCalls(): Promise<FunctionCall[]> {
    return Promise.resolve([]);
  }

  getFeePayer(): Promise<AztecAddress> {
    return Promise.resolve(this.sender);
  }
}
