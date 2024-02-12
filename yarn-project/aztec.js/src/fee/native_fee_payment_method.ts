import { FunctionCall } from '@aztec/circuit-types';
import { FunctionData } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { FeePaymentMethod } from './fee_payment_method.js';

/**
 * Pay fee directly in the native gas token.
 */
export class NativeFeePaymentMethod implements FeePaymentMethod {
  // TODO(fees) replace this with the address of the gas token when that's deployed.
  static #GAS_TOKEN = AztecAddress.ZERO;

  constructor() {}

  /**
   * Gets the native gas asset used to pay the fee.
   * @returns The asset used to pay the fee.
   */
  getAsset() {
    return NativeFeePaymentMethod.#GAS_TOKEN;
  }

  /**
   * The contract responsible for fee payment. This will be the same as the asset.
   * @returns The contract address responsible for holding the fee payment.
   */
  getPaymentContract() {
    return NativeFeePaymentMethod.#GAS_TOKEN;
  }

  /**
   * Fee payments in the native gas token are always public.
   * @returns false
   */
  isPrivateFeePayment(): boolean {
    return false;
  }

  /**
   * Creates a function call to pay the fee in gas token..
   * @param feeLimit - The maximum fee to be paid in gas token.
   * @returns A function call
   */
  getFunctionCalls(feeLimit: Fr): FunctionCall[] {
    return [
      {
        to: NativeFeePaymentMethod.#GAS_TOKEN,
        functionData: new FunctionData(FunctionSelector.fromSignature('check_balance(Field)'), false, false, false),
        args: [feeLimit],
      },
      {
        to: NativeFeePaymentMethod.#GAS_TOKEN,
        functionData: new FunctionData(FunctionSelector.fromSignature('pay_fee(Field)'), false, false, false),
        args: [feeLimit],
      },
    ];
  }
}
