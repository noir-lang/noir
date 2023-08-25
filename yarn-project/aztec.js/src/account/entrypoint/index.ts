import { AztecAddress } from '@aztec/circuits.js';
import { FunctionCall, TxExecutionRequest } from '@aztec/types';

export * from './entrypoint_collection.js';
export * from './entrypoint_payload.js';
export * from './entrypoint_utils.js';
export * from './single_key_account_entrypoint.js';
export * from './stored_key_account_entrypoint.js';

/** Options for creating a tx request out of a set of function calls. */
export type CreateTxRequestOpts = {
  /** Origin of the tx. Needs to be an address managed by this account. */
  origin?: AztecAddress;
};

// docs:start:entrypoint-interface
/**
 * Represents a transaction entrypoint in an account contract.
 * Knows how to assemble a transaction execution request given a set of function calls.
 */
export interface Entrypoint {
  // docs:start:entrypoint-interface
  /**
   * Generates an authenticated request out of set of intents
   * @param executions - The execution intents to be run.
   * @param opts - Options.
   * @returns The authenticated transaction execution request.
   */
  createTxExecutionRequest(executions: FunctionCall[], opts?: CreateTxRequestOpts): Promise<TxExecutionRequest>;
  // docs:end:entrypoint-interface
}
// docs:end:entrypoint-interface
