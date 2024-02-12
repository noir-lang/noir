import { FunctionCall } from '@aztec/circuit-types';
import { FunctionData } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { FeePaymentMethod } from './fee_payment_method.js';

/**
 * Holds information about how the fee for a transaction is to be paid.
 */
export class GenericFeePaymentMethod implements FeePaymentMethod {
  constructor(
    /**
     * The asset used to pay the fee.
     */
    private asset: AztecAddress,
    /**
     * Address which will hold the fee payment.
     */
    private paymentContract: AztecAddress,
    /**
     * Whether the fee payment is private
     */
    private privatePayment: boolean,
  ) {}

  /**
   * The asset used to pay the fee.
   * @returns The asset used to pay the fee.
   */
  getAsset() {
    return this.asset;
  }

  /**
   * The address which will facilitate the fee payment.
   * @returns The contract address responsible for holding the fee payment.
   */
  getPaymentContract() {
    return this.paymentContract;
  }

  /**
   * The fee payment function selector on the fee payment contract.
   * @returns The fee payment function selector on the fee payment contract.
   */
  #getFeePaymentEntrypoint() {
    return this.privatePayment
      ? FunctionSelector.fromSignature('prepare_fee_private(Field, (Field))')
      : FunctionSelector.fromSignature('prepare_fee_public(Field, (Field))');
  }

  /**
   * Whether the fee payment is private or not
   * @returns Whether the fee payment is private or not
   */
  isPrivateFeePayment(): boolean {
    return this.privatePayment;
  }

  /**
   * Creates a function call to pay the fee in the given asset.
   * @param maxFee - The maximum fee to be paid in the given asset.
   * @returns The function call to pay the fee.
   */
  getFunctionCalls(maxFee: Fr): FunctionCall[] {
    return [
      // TODO(fees) set up auth witnesses
      {
        to: this.getPaymentContract(),
        functionData: new FunctionData(this.#getFeePaymentEntrypoint(), false, true, false),
        args: [maxFee, this.asset],
      },
    ];
  }

  static empty(): GenericFeePaymentMethod {
    return new GenericFeePaymentMethod(AztecAddress.ZERO, AztecAddress.ZERO, false);
  }
}
