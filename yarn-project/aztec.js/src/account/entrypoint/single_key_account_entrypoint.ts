import { AztecAddress, FunctionData, GrumpkinPrivateKey, PartialAddress, TxContext } from '@aztec/circuits.js';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { ContractAbi, encodeArguments } from '@aztec/foundation/abi';
import { FunctionCall, PackedArguments, TxExecutionRequest } from '@aztec/types';

import SchnorrSingleKeyAccountContractAbi from '../../abis/schnorr_single_key_account_contract.json' assert { type: 'json' };
import { generatePublicKey } from '../../index.js';
import { DEFAULT_CHAIN_ID, DEFAULT_VERSION } from '../../utils/defaults.js';
import { buildPayload, hashPayload } from './entrypoint_payload.js';
import { CreateTxRequestOpts, Entrypoint } from './index.js';

/**
 * Account contract implementation that uses a single key for signing and encryption. This public key is not
 * stored in the contract, but rather verified against the contract address. Note that this approach is not
 * secure and should not be used in real use cases.
 */
export class SingleKeyAccountEntrypoint implements Entrypoint {
  constructor(
    private address: AztecAddress,
    private partialAddress: PartialAddress,
    private privateKey: GrumpkinPrivateKey,
    private chainId: number = DEFAULT_CHAIN_ID,
    private version: number = DEFAULT_VERSION,
  ) {}

  async createTxExecutionRequest(
    executions: FunctionCall[],
    opts: CreateTxRequestOpts = {},
  ): Promise<TxExecutionRequest> {
    if (opts.origin && !opts.origin.equals(this.address)) {
      throw new Error(`Sender ${opts.origin.toString()} does not match account address ${this.address.toString()}`);
    }

    const { payload, packedArguments: callsPackedArguments } = await buildPayload(executions);
    const message = await hashPayload(payload);

    const signer = await Schnorr.new();
    const signature = signer.constructSignature(message, this.privateKey).toBuffer();
    const publicKey = await generatePublicKey(this.privateKey);
    const args = [payload, publicKey.toBuffer(), signature, this.partialAddress];
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
    // We use the SchnorrSingleKeyAccountContract because it implements the interface we need, but ideally
    // we should have an interface that defines the entrypoint for SingleKeyAccountContracts and
    // load the abi from it.
    const abi = (SchnorrSingleKeyAccountContractAbi as any as ContractAbi).functions.find(f => f.name === 'entrypoint');
    if (!abi) throw new Error(`Entrypoint abi for account contract not found`);
    return abi;
  }
}
