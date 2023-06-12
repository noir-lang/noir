import { TxContext } from '@aztec/circuits.js';
import { ExecutionRequest, SignedTxExecutionRequest } from '@aztec/types';

/** Represents an implementation for a user account contract. Knows how to encode and sign a tx for that particular implementation. */
export interface AccountImplementation {
  /** Creates a tx to be sent from a given account contract given a set of execution requests. */
  createAuthenticatedTxRequest(executions: ExecutionRequest[], txContext: TxContext): Promise<SignedTxExecutionRequest>;
}
