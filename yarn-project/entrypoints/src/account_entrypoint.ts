import { type AuthWitnessProvider } from '@aztec/aztec.js/account';
import { type EntrypointInterface, EntrypointPayload, type ExecutionRequestInit } from '@aztec/aztec.js/entrypoint';
import { PackedValues, TxExecutionRequest } from '@aztec/circuit-types';
import { type AztecAddress, FunctionData, GasSettings, TxContext } from '@aztec/circuits.js';
import { type FunctionAbi, encodeArguments } from '@aztec/foundation/abi';

import { DEFAULT_CHAIN_ID, DEFAULT_VERSION } from './constants.js';

/**
 * Implementation for an entrypoint interface that follows the default entrypoint signature
 * for an account, which accepts an AppPayload and a FeePayload as defined in noir-libs/aztec-noir/src/entrypoint module
 */
export class DefaultAccountEntrypoint implements EntrypointInterface {
  constructor(
    private address: AztecAddress,
    private auth: AuthWitnessProvider,
    private chainId: number = DEFAULT_CHAIN_ID,
    private version: number = DEFAULT_VERSION,
  ) {}

  async createTxExecutionRequest(exec: ExecutionRequestInit): Promise<TxExecutionRequest> {
    const { calls, fee } = exec;
    const appPayload = EntrypointPayload.fromAppExecution(calls);
    const feePayload = await EntrypointPayload.fromFeeOptions(fee);

    const abi = this.getEntrypointAbi();
    const entrypointPackedArgs = PackedValues.fromValues(encodeArguments(abi, [appPayload, feePayload]));
    const gasSettings = exec.fee?.gasSettings ?? GasSettings.default();

    const appAuthWitness = await this.auth.createAuthWit(appPayload.hash());
    const feeAuthWitness = await this.auth.createAuthWit(feePayload.hash());

    const txRequest = TxExecutionRequest.from({
      argsHash: entrypointPackedArgs.hash,
      origin: this.address,
      functionData: FunctionData.fromAbi(abi),
      txContext: new TxContext(this.chainId, this.version, gasSettings),
      packedArguments: [...appPayload.packedArguments, ...feePayload.packedArguments, entrypointPackedArgs],
      authWitnesses: [appAuthWitness, feeAuthWitness],
    });

    return txRequest;
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
        {
          name: 'fee_payload',
          type: {
            kind: 'struct',
            path: 'authwit::entrypoint::fee::FeePayload',
            fields: [
              {
                name: 'function_calls',
                type: {
                  kind: 'array',
                  length: 2,
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
