import { type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress, type GasSettings } from '@aztec/circuits.js';
import { FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { GasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { type FeePaymentMethod } from './fee_payment_method.js';

/**
 * Pay fee directly in the native gas token.
 */
export class NativeFeePaymentMethod implements FeePaymentMethod {
  #gasTokenAddress: AztecAddress;

  constructor() {
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
   * The contract responsible for fee payment. This will be the same as the asset.
   * @returns The contract address responsible for holding the fee payment.
   */
  getPaymentContract() {
    return this.#gasTokenAddress;
  }

  /**
   * Fee payments in the native gas token are always public.
   * @returns false
   */
  isPrivateFeePayment(): boolean {
    return false;
  }

  /**
   * Creates a function call to pay the fee in gas token.
   * @param gasSettings - The gas settings.
   * @returns A function call
   */
  getFunctionCalls(gasSettings: GasSettings): Promise<FunctionCall[]> {
    return Promise.resolve([
      {
        name: 'pay_fee',
        to: this.#gasTokenAddress,
        selector: FunctionSelector.fromSignature('pay_fee(Field)'),
        type: FunctionType.PUBLIC,
        isStatic: false,
        args: [gasSettings.getFeeLimit()],
        returnTypes: [],
      },
    ]);
  }
}
