import { type EntrypointInterface } from '@aztec/aztec.js/entrypoint';
import { type FunctionCall, PackedArguments, TxExecutionRequest } from '@aztec/circuit-types';
import { type AztecAddress, FunctionData, TxContext } from '@aztec/circuits.js';
import { type FunctionAbi, encodeArguments } from '@aztec/foundation/abi';
import { getCanonicalMultiCallEntrypointAddress } from '@aztec/protocol-contracts/multi-call-entrypoint';

import { DEFAULT_CHAIN_ID, DEFAULT_VERSION } from './constants.js';
import { buildAppPayload } from './entrypoint_payload.js';

/**
 * Implementation for an entrypoint interface that can execute multiple function calls in a single transaction
 */
export class DefaultMultiCallEntrypoint implements EntrypointInterface {
  constructor(
    private address: AztecAddress = getCanonicalMultiCallEntrypointAddress(),
    private chainId: number = DEFAULT_CHAIN_ID,
    private version: number = DEFAULT_VERSION,
  ) {}

  createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest> {
    const { payload: appPayload, packedArguments: appPackedArguments } = buildAppPayload(executions);

    const abi = this.getEntrypointAbi();
    const entrypointPackedArgs = PackedArguments.fromArgs(encodeArguments(abi, [appPayload]));

    const txRequest = TxExecutionRequest.from({
      argsHash: entrypointPackedArgs.hash,
      origin: this.address,
      functionData: FunctionData.fromAbi(abi),
      txContext: TxContext.empty(this.chainId, this.version),
      packedArguments: [...appPackedArguments, entrypointPackedArgs],
      authWitnesses: [],
    });

    return Promise.resolve(txRequest);
  }

  private getEntrypointAbi() {
    return {
      name: 'entrypoint',
      isInitializer: false,
      functionType: 'secret',
      isInternal: false,
      parameters: [
        {
          name: 'app_payload',
          type: {
            kind: 'struct',
            path: 'authwit::entrypoint::app::AppPayload',
            fields: [
              {
                name: 'function_calls',
                type: {
                  kind: 'array',
                  length: 4,
                  type: {
                    kind: 'struct',
                    path: 'authwit::entrypoint::function_call::FunctionCall',
                    fields: [
                      { name: 'args_hash', type: { kind: 'field' } },
                      {
                        name: 'function_selector',
                        type: {
                          kind: 'struct',
                          path: 'authwit::aztec::protocol_types::abis::function_selector::FunctionSelector',
                          fields: [{ name: 'inner', type: { kind: 'integer', sign: 'unsigned', width: 32 } }],
                        },
                      },
                      {
                        name: 'target_address',
                        type: {
                          kind: 'struct',
                          path: 'authwit::aztec::protocol_types::address::AztecAddress',
                          fields: [{ name: 'inner', type: { kind: 'field' } }],
                        },
                      },
                      { name: 'is_public', type: { kind: 'boolean' } },
                    ],
                  },
                },
              },
              { name: 'nonce', type: { kind: 'field' } },
            ],
          },
          visibility: 'public',
        },
      ],
      returnTypes: [],
    } as FunctionAbi;
  }
}
