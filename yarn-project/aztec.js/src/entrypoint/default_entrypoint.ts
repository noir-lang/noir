import { PackedValues, TxExecutionRequest } from '@aztec/circuit-types';
import { GasSettings, TxContext } from '@aztec/circuits.js';
import { FunctionType } from '@aztec/foundation/abi';

import { type EntrypointInterface, type ExecutionRequestInit } from './entrypoint.js';

/**
 * Default implementation of the entrypoint interface. It calls a function on a contract directly
 */
export class DefaultEntrypoint implements EntrypointInterface {
  constructor(private chainId: number, private protocolVersion: number) {}

  createTxExecutionRequest(exec: ExecutionRequestInit): Promise<TxExecutionRequest> {
    const { calls, authWitnesses = [], packedArguments = [] } = exec;

    if (calls.length > 1) {
      throw new Error(`Expected a single call, got ${calls.length}`);
    }

    const call = calls[0];

    if (call.type !== FunctionType.PRIVATE) {
      throw new Error('Public entrypoints are not allowed');
    }

    const entrypointPackedValues = PackedValues.fromValues(call.args);
    const gasSettings = exec.fee?.gasSettings ?? GasSettings.default();
    const txContext = new TxContext(this.chainId, this.protocolVersion, gasSettings);
    return Promise.resolve(
      new TxExecutionRequest(
        call.to,
        call.selector,
        entrypointPackedValues.hash,
        txContext,
        [...packedArguments, entrypointPackedValues],
        authWitnesses,
      ),
    );
  }
}
