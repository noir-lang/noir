import { type AuthWitness, type FunctionCall, type PackedValues, type TxExecutionRequest } from '@aztec/circuit-types';
import { type Fr } from '@aztec/circuits.js';

import { EntrypointPayload, type FeeOptions, computeCombinedPayloadHash } from './payload.js';

export { EntrypointPayload, FeeOptions, computeCombinedPayloadHash };

export { DefaultEntrypoint } from './default_entrypoint.js';
export { DefaultMultiCallEntrypoint } from './default_multi_call_entrypoint.js';

/** Encodes the calls to be done in a transaction. */
export type ExecutionRequestInit = {
  /** The function calls to be executed. */
  calls: FunctionCall[];
  /** Any transient auth witnesses needed for this execution */
  authWitnesses?: AuthWitness[];
  /** Any transient packed arguments for this execution */
  packedArguments?: PackedValues[];
  /** How the fee is going to be payed */
  fee?: FeeOptions;
  /** An optional nonce. Used to repeat a previous tx with a higher fee so that the first one is cancelled */
  nonce?: Fr;
  /** Whether the transaction can be cancelled. If true, an extra nullifier will be emitted: H(nonce, GENERATOR_INDEX__TX_NULLIFIER) */
  cancellable?: boolean;
};

/** Creates transaction execution requests out of a set of function calls. */
export interface EntrypointInterface {
  /**
   * Generates an execution request out of set of function calls.
   * @param execution - The execution intents to be run.
   * @returns The authenticated transaction execution request.
   */
  createTxExecutionRequest(execution: ExecutionRequestInit): Promise<TxExecutionRequest>;
}
