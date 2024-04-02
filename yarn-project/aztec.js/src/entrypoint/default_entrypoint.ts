import { type FunctionCall, PackedArguments, TxExecutionRequest } from '@aztec/circuit-types';
import { TxContext } from '@aztec/circuits.js';

import { type EntrypointInterface } from './entrypoint.js';

/**
 * Default implementation of the entrypoint interface. It calls a function on a contract directly
 */
export class DefaultEntrypoint implements EntrypointInterface {
  constructor(private chainId: number, private protocolVersion: number) {}

  createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest> {
    const [execution] = executions;
    const packedArguments = PackedArguments.fromArgs(execution.args);
    const txContext = TxContext.empty(this.chainId, this.protocolVersion);
    return Promise.resolve(
      new TxExecutionRequest(
        execution.to,
        execution.functionData,
        packedArguments.hash,
        txContext,
        [packedArguments],
        [],
      ),
    );
  }
}
