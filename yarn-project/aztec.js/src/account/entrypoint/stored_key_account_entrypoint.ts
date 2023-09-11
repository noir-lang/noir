import { AztecAddress, FunctionData, TxContext } from '@aztec/circuits.js';
import { Signature } from '@aztec/circuits.js/barretenberg';
import { ContractAbi, encodeArguments } from '@aztec/foundation/abi';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { FunctionCall, PackedArguments, TxExecutionRequest } from '@aztec/types';

import EcdsaAccountContractAbi from '../../abis/ecdsa_account_contract.json' assert { type: 'json' };
import { DEFAULT_CHAIN_ID, DEFAULT_VERSION } from '../../utils/defaults.js';
import { buildPayload, hashPayload } from './entrypoint_payload.js';
import { Entrypoint } from './index.js';

/**
 * Account contract implementation that keeps a signing public key in storage, and is retrieved on
 * every new request in order to validate the payload signature.
 */
export class StoredKeyAccountEntrypoint implements Entrypoint {
  private log: DebugLogger;

  constructor(
    private address: AztecAddress,
    private sign: (msg: Buffer) => Signature,
    private chainId: number = DEFAULT_CHAIN_ID,
    private version: number = DEFAULT_VERSION,
  ) {
    this.log = createDebugLogger('aztec:client:accounts:stored_key');
  }

  async createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest> {
    const { payload, packedArguments: callsPackedArguments } = await buildPayload(executions);
    const message = await hashPayload(payload);
    const signature = this.sign(message).toBuffer();
    this.log(`Signed challenge ${message.toString('hex')} as ${signature.toString('hex')}`);

    const args = [payload, signature];
    const abi = this.getEntrypointAbi();
    const packedArgs = await PackedArguments.fromArgs(encodeArguments(abi, args));
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
