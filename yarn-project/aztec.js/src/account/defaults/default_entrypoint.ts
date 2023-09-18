import { AztecAddress, Fr, FunctionData, TxContext } from '@aztec/circuits.js';
import { FunctionAbiHeader, encodeArguments } from '@aztec/foundation/abi';
import { FunctionCall, PackedArguments, TxExecutionRequest } from '@aztec/types';

import { DEFAULT_CHAIN_ID, DEFAULT_VERSION } from '../../utils/defaults.js';
import { AuthWitnessProvider, EntrypointInterface } from '../interface.js';
import { buildPayload, hashPayload } from './entrypoint_payload.js';

/**
 * Implementation for an entrypoint interface that follows the default entrypoint signature
 * for an account, which accepts an EntrypointPayload as defined in noir-libs/aztec-noir/src/entrypoint.nr.
 */
export class DefaultAccountEntrypoint implements EntrypointInterface {
  constructor(
    private address: AztecAddress,
    private auth: AuthWitnessProvider,
    private chainId: number = DEFAULT_CHAIN_ID,
    private version: number = DEFAULT_VERSION,
  ) {}

  async createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest> {
    const { payload, packedArguments: callsPackedArguments } = await buildPayload(executions);
    const abi = this.getEntrypointAbi();
    const packedArgs = await PackedArguments.fromArgs(encodeArguments(abi, [payload]));
    const message = Fr.fromBuffer(await hashPayload(payload));
    const authWitness = await this.auth.createAuthWitness(message);
    const txRequest = TxExecutionRequest.from({
      argsHash: packedArgs.hash,
      origin: this.address,
      functionData: FunctionData.fromAbi(abi),
      txContext: TxContext.empty(this.chainId, this.version),
      packedArguments: [...callsPackedArguments, packedArgs],
      authWitnesses: [authWitness],
    });

    return txRequest;
  }

  private getEntrypointAbi() {
    return {
      name: 'entrypoint',
      functionType: 'secret',
      isInternal: false,
      parameters: [
        {
          name: 'payload',
          type: {
            kind: 'struct',
            path: 'aztec::entrypoint::EntrypointPayload',
            fields: [
              {
                name: 'function_calls',
                type: {
                  kind: 'array',
                  length: 4,
                  type: {
                    kind: 'struct',
                    path: 'aztec::entrypoint::FunctionCall',
                    fields: [
                      {
                        name: 'args_hash',
                        type: {
                          kind: 'field',
                        },
                      },
                      {
                        name: 'function_selector',
                        type: {
                          kind: 'field',
                        },
                      },
                      {
                        name: 'target_address',
                        type: {
                          kind: 'field',
                        },
                      },
                      {
                        name: 'is_public',
                        type: {
                          kind: 'boolean',
                        },
                      },
                    ],
                  },
                },
              },
              {
                name: 'nonce',
                type: {
                  kind: 'field',
                },
              },
            ],
          },
          visibility: 'public',
        },
      ],
      returnTypes: [],
    } as FunctionAbiHeader;
  }
}
