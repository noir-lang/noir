import { AztecAddress, TxContext } from '@aztec/circuits.js';
import { ExecutionRequest, TxExecutionRequest } from '@aztec/types';

export * from './single_key_account_contract.js';
export * from './stored_key_account_contract.js';
export * from './account_collection.js';

/** Represents an implementation for a user account contract. Knows how to encode and sign a tx for that particular implementation. */
export interface AccountImplementation {
  /**
   * Returns the address for the account contract used by this implementation.
   * @returns The address.
   */
  getAddress(): AztecAddress;

  /**
   * Generates an authenticated request out of set of intents
   * @param executions - The execution intent to be authenticated.
   * @param txContext - The tx context under with the execution is to be made.
   * @returns The authenticated transaction execution request.
   */
  createAuthenticatedTxRequest(executions: ExecutionRequest[], txContext: TxContext): Promise<TxExecutionRequest>;
}
