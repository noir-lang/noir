import { AztecAddress } from '@aztec/circuits.js';
import { FunctionCall, TxExecutionRequest } from '@aztec/types';

export * from './account_collection.js';
export * from './single_key_account_contract.js';
export * from './stored_key_account_contract.js';

/** Options for creating a tx request out of a set of function calls. */
export type CreateTxRequestOpts = {
  /** Origin of the tx. Needs to be an address managed by this account. */
  origin?: AztecAddress;
};

/** Represents an implementation for a user account contract. Knows how to encode and sign a tx for that particular implementation. */
export interface AccountImplementation {
  /**
   * Returns the address for the account contract used by this implementation.
   * @returns The address.
   */
  getAddress(): AztecAddress;

  /**
   * Generates an authenticated request out of set of intents
   * @param executions - The execution intents to be run.
   * @param opts - Options.
   * @returns The authenticated transaction execution request.
   */
  createTxExecutionRequest(executions: FunctionCall[], opts?: CreateTxRequestOpts): Promise<TxExecutionRequest>;
}
