import { AuthWitness, CompleteAddress, FunctionCall, TxExecutionRequest } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
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
   * Computes an authentication witness from either a message hash or an intent (caller and an action).
   * If a message hash is provided, it will create a witness for that directly.
   * Otherwise, it will compute the message hash using the caller and the action of the intent.
   * @param messageHashOrIntent - The message hash or the intent (caller and action) to approve
   * @param chainId - The chain id for the message, will default to the current chain id
   * @param version - The version for the message, will default to the current protocol version
   * @returns The authentication witness
   */
  createAuthWit(
    messageHashOrIntent:
      | Fr
      | Buffer
      | {
          /** The caller to approve  */
          caller: AztecAddress;
          /** The action to approve */
          action: ContractFunctionInteraction | FunctionCall;
          /** The chain id to approve */
          chainId?: Fr;
          /** The version to approve  */
          version?: Fr;
        },
  ): Promise<AuthWitness>;
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
  /** Returns the complete address for this account. */
  getCompleteAddress(): CompleteAddress;

  /** Returns the address for this account. */
  getAddress(): AztecAddress;

  /** Returns the chain id for this account */
  getChainId(): Fr;

  /** Returns the rollup version for this account */
  getVersion(): Fr;
}
// docs:end:account-interface
