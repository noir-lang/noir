import { type FunctionCall } from '@aztec/circuit-types';
import { type GasSettings } from '@aztec/circuits.js';
import { type AztecAddress } from '@aztec/foundation/aztec-address';

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
   * @param gasSettings - The gas limits and max fees.
   * @returns The function call to pay the fee.
   */
  getFunctionCalls(gasSettings: GasSettings): Promise<FunctionCall[]>;
}
