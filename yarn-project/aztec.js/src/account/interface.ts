import { AuthWitness, CompleteAddress, FunctionCall, TxExecutionRequest } from '@aztec/circuit-types';
import { Fr } from '@aztec/foundation/fields';

import { FeePaymentMethod } from '../fee/fee_payment_method.js';

/**
 * Fee payment options for a transaction.
 */
export type FeeOptions = {
  /** The fee payment method to use */
  paymentMethod: FeePaymentMethod;
  /** The fee limit to pay */
  maxFee: bigint | number | Fr;
};

// docs:start:account-interface
/** Creates authorization witnesses. */
export interface AuthWitnessProvider {
  /**
   * Create an authorization witness for the given message.
   * @param message - Message to authorize.
   */
  createAuthWitness(message: Fr): Promise<AuthWitness>;
}

/** Creates transaction execution requests out of a set of function calls. */
export interface EntrypointInterface {
  /**
   * Generates an authenticated request out of set of function calls.
   * @param executions - The execution intents to be run.
   * @param feeOpts - The fee to be paid for the transaction.
   * @returns The authenticated transaction execution request.
   */
  createTxExecutionRequest(executions: FunctionCall[], feeOpts?: FeeOptions): Promise<TxExecutionRequest>;
}

/**
 * Handler for interfacing with an account. Knows how to create transaction execution
 * requests and authorize actions for its corresponding account.
 */
export interface AccountInterface extends AuthWitnessProvider, EntrypointInterface {
  /**
   * Returns the complete address for this account.
   */
  getCompleteAddress(): CompleteAddress;
}
// docs:end:account-interface
