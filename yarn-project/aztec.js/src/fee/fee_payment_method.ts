import { type FunctionCall } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';

/**
 * Holds information about how the fee for a transaction is to be paid.
 */
export interface FeePaymentMethod {
  /**
   * The asset used to pay the fee.
   */
  getAsset(): AztecAddress;
  /**
   * Address which will hold the fee payment.
   */
  getPaymentContract(): AztecAddress;

  /**
   * Creates a function call to pay the fee in the given asset.
   * TODO(fees) replace maxFee with gas limits
   * @param maxFee - The maximum fee to be paid in the given asset.
   * @returns The function call to pay the fee.
   */
  getFunctionCalls(maxFee: Fr): Promise<FunctionCall[]>;
}
