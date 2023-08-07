import { AztecAddress, CircuitsWasm, FunctionData, PrivateKey, TxContext } from '@aztec/circuits.js';
import { Signer } from '@aztec/circuits.js/barretenberg';
import { ContractAbi, encodeArguments } from '@aztec/foundation/abi';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { FunctionCall, PackedArguments, TxExecutionRequest } from '@aztec/types';

import partition from 'lodash.partition';

import EcdsaAccountContractAbi from '../../abis/ecdsa_account_contract.json' assert { type: 'json' };
import { DEFAULT_CHAIN_ID, DEFAULT_VERSION } from '../../utils/defaults.js';
import { buildPayload, hashPayload } from './entrypoint_payload.js';
import { CreateTxRequestOpts, Entrypoint } from './index.js';

/**
 * Account contract implementation that keeps a signing public key in storage, and is retrieved on
 * every new request in order to validate the payload signature.
 */
export class StoredKeyAccountEntrypoint implements Entrypoint {
  private log: DebugLogger;

  constructor(
    private address: AztecAddress,
    private privateKey: PrivateKey,
    private signer: Signer,
    private chainId: number = DEFAULT_CHAIN_ID,
    private version: number = DEFAULT_VERSION,
  ) {
    this.log = createDebugLogger('aztec:client:accounts:stored_key');
  }

  async createTxExecutionRequest(
    executions: FunctionCall[],
    opts: CreateTxRequestOpts = {},
  ): Promise<TxExecutionRequest> {
    if (opts.origin && !opts.origin.equals(this.address)) {
      throw new Error(`Sender ${opts.origin.toString()} does not match account address ${this.address.toString()}`);
    }

    const [privateCalls, publicCalls] = partition(executions, exec => exec.functionData.isPrivate);
    const wasm = await CircuitsWasm.get();
    const { payload, packedArguments: callsPackedArguments } = await buildPayload(privateCalls, publicCalls);
    const message = hashPayload(payload, wasm);
    const signature = this.signer.constructSignature(message, this.privateKey).toBuffer();
    this.log(`Signed challenge ${message.toString('hex')} as ${signature.toString('hex')}`);

    const args = [payload, signature];
    const abi = this.getEntrypointAbi();
    const packedArgs = await PackedArguments.fromArgs(encodeArguments(abi, args), wasm);
    const txRequest = TxExecutionRequest.from({
      argsHash: packedArgs.hash,
      origin: this.address,
      functionData: FunctionData.fromAbi(abi),
      txContext: TxContext.empty(this.chainId, this.version),
      packedArguments: [...callsPackedArguments, packedArgs],
    });

    return txRequest;
  }

  private getEntrypointAbi() {
    // We use the EcdsaAccountContractAbi because it implements the interface we need, but ideally
    // we should have an interface that defines the entrypoint for StoredKeyAccountContracts and
    // load the abi from it.
    const abi = (EcdsaAccountContractAbi as any as ContractAbi).functions.find(f => f.name === 'entrypoint');
    if (!abi) throw new Error(`Entrypoint abi for account contract not found`);
    return abi;
  }
}
