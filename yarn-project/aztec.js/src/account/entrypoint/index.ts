import { FunctionCall, TxExecutionRequest } from '@aztec/types';

export * from './auth_witness_account_entrypoint.js';
export * from './entrypoint_payload.js';
export * from './entrypoint_utils.js';
export * from './single_key_account_entrypoint.js';
export * from './stored_key_account_entrypoint.js';

// docs:start:entrypoint-interface
/**
 * Represents a transaction entrypoint in an account contract.
 * Knows how to assemble a transaction execution request given a set of function calls.
 */
export interface Entrypoint {
  /**
   * Generates an authenticated request out of set of intents
   * @param executions - The execution intents to be run.
   * @param opts - Options.
   * @returns The authenticated transaction execution request.
   */
  createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest>;
}
// docs:end:entrypoint-interface
