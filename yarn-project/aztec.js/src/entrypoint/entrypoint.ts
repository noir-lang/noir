import { type FunctionCall, type TxExecutionRequest } from '@aztec/circuit-types';
import { type Fr } from '@aztec/foundation/fields';

import { type FeePaymentMethod } from '../fee/fee_payment_method.js';

/**
 * Fee payment options for a transaction.
 */
export type FeeOptions = {
  /** The fee payment method to use */
  paymentMethod: FeePaymentMethod;
  /** The fee limit to pay */
  maxFee: bigint | number | Fr;
};

/** Creates transaction execution requests out of a set of function calls. */
export interface EntrypointInterface {
  /**
   * Generates an execution request out of set of function calls.
   * @param executions - The execution intents to be run.
   * @param feeOpts - The fee to be paid for the transaction.
   * @returns The authenticated transaction execution request.
   */
  createTxExecutionRequest(executions: FunctionCall[], feeOpts?: FeeOptions): Promise<TxExecutionRequest>;
}
