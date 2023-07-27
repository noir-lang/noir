import { AztecAddress, CircuitsWasm, FunctionData, PrivateKey, TxContext } from '@aztec/circuits.js';
import { Signer } from '@aztec/circuits.js/barretenberg';
import { ContractAbi, encodeArguments, generateFunctionSelector } from '@aztec/foundation/abi';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { ExecutionRequest, PackedArguments, TxExecutionRequest } from '@aztec/types';

import partition from 'lodash.partition';

import EcdsaAccountContractAbi from '../abis/ecdsa_account_contract.json' assert { type: 'json' };
import { buildPayload, hashPayload } from './entrypoint_payload.js';
import { AccountImplementation } from './index.js';

/**
 * Account contract implementation that keeps a signing public key in storage, and is retrieved on
 * every new request in order to validate the payload signature.
 */
export class StoredKeyAccountContract implements AccountImplementation {
  private log: DebugLogger;

  constructor(private address: AztecAddress, private privateKey: PrivateKey, private signer: Signer) {
    this.log = createDebugLogger('aztec:client:accounts:stored_key');
  }

  getAddress(): AztecAddress {
    return this.address;
  }

  async createAuthenticatedTxRequest(
    executions: ExecutionRequest[],
    txContext: TxContext,
  ): Promise<TxExecutionRequest> {
    this.checkSender(executions);
    this.checkIsNotDeployment(txContext);

    const [privateCalls, publicCalls] = partition(executions, exec => exec.functionData.isPrivate);
    const wasm = await CircuitsWasm.get();
    const { payload, packedArguments: callsPackedArguments } = await buildPayload(privateCalls, publicCalls);
    const hash = hashPayload(payload);
    const signature = this.signer.constructSignature(hash, this.privateKey).toBuffer();
    this.log(`Signed challenge ${hash.toString('hex')} as ${signature.toString('hex')}`);

    const args = [payload, signature];
    const abi = this.getEntrypointAbi();
    const selector = generateFunctionSelector(abi.name, abi.parameters);
    const packedArgs = await PackedArguments.fromArgs(encodeArguments(abi, args), wasm);
    const txRequest = TxExecutionRequest.from({
      argsHash: packedArgs.hash,
      origin: this.address,
      functionData: new FunctionData(selector, abi.isInternal, true, false),
      txContext,
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

  private checkIsNotDeployment(txContext: TxContext) {
    if (txContext.isContractDeploymentTx) {
      throw new Error(`Cannot yet deploy contracts from an account contract`);
    }
  }

  private checkSender(executions: ExecutionRequest[]) {
    const wrongSender = executions.find(e => !e.from.equals(this.address));
    if (wrongSender) {
      throw new Error(
        `Sender ${wrongSender.from.toString()} does not match account address ${this.address.toString()}`,
      );
    }
  }
}
