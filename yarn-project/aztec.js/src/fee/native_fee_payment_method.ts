import { type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { GasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Pay fee directly in the native gas token.
 */
export class NativeFeePaymentMethod implements FeePaymentMethod {
  #gasTokenAddress: AztecAddress;

  constructor(private sender: AztecAddress) {
    this.#gasTokenAddress = GasTokenAddress;
  }

  /**
   * Gets the native gas asset used to pay the fee.
   * @returns The asset used to pay the fee.
   */
  getAsset() {
    return this.#gasTokenAddress;
  }

  /**
   * Creates a function call to pay the fee in gas token.
   * @returns A function call
   */
  getFunctionCalls(): Promise<FunctionCall[]> {
    return Promise.resolve([]);
  }

  getFeePayer(): Promise<AztecAddress> {
    return Promise.resolve(this.sender);
  }
}
