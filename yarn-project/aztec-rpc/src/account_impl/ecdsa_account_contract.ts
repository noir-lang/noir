import { encodeArguments } from '@aztec/acir-simulator';
import { ARGS_LENGTH, AztecAddress, Fr, FunctionData, TxContext } from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { sha256 } from '@aztec/foundation/crypto';
import { KeyStore, PublicKey } from '@aztec/key-store';
import { AccountContractAbi } from '@aztec/noir-contracts/examples';
import { ExecutionRequest, TxExecutionRequest } from '@aztec/types';
import partition from 'lodash.partition';
import times from 'lodash.times';
import { generateFunctionSelector } from '../index.js';
import { AccountImplementation } from './index.js';

/**
 * Account backed by an account contract that uses ECDSA signatures for authorization.
 */
export class EcdsaAccountContract implements AccountImplementation {
  constructor(private address: AztecAddress, private pubKey: PublicKey, private keyStore: KeyStore) {}

  async createAuthenticatedTxRequest(
    executions: ExecutionRequest[],
    txContext: TxContext,
  ): Promise<TxExecutionRequest> {
    this.checkSender(executions);
    this.checkIsNotDeployment(txContext);

    const [privateCalls, publicCalls] = partition(executions, exec => exec.functionData.isPrivate).map(execs =>
      execs.map(exec => ({
        args: exec.args,
        selector: exec.functionData.functionSelectorBuffer,
        target: exec.to,
      })),
    );

    const payload = buildPayload(privateCalls, publicCalls);
    const hash = hashPayload(payload);
    const signature = await this.keyStore.ecdsaSign(hash, this.pubKey);
    const signatureAsFrArray = Array.from(signature.toBuffer()).map(byte => new Fr(byte));
    const args = [payload, signatureAsFrArray];
    const abi = this.getEntrypointAbi();
    const selector = generateFunctionSelector(abi.name, abi.parameters);
    const txRequest = TxExecutionRequest.from({
      args: encodeArguments(abi, args),
      origin: this.address,
      functionData: new FunctionData(selector, true, false),
      txContext: TxContext.empty(),
    });

    return txRequest;
  }

  private getEntrypointAbi() {
    const abi = AccountContractAbi.functions.find(f => f.name === 'entrypoint');
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

const ACCOUNT_MAX_PRIVATE_CALLS = 1;
const ACCOUNT_MAX_PUBLIC_CALLS = 1;

/** A call to a function in a noir contract */
type FunctionCall = {
  /** The encoded arguments */
  args: Fr[];
  /** The function selector */
  selector: Buffer;
  /** The address of the contract */
  target: AztecAddress;
};

/** Encoded payload for the account contract entrypoint */
type EntrypointPayload = {
  // eslint-disable-next-line camelcase
  /** Concatenated arguments for every call */
  flattened_args: Fr[];
  // eslint-disable-next-line camelcase
  /** Concatenated selectors for every call */
  flattened_selectors: Fr[];
  // eslint-disable-next-line camelcase
  /** Concatenated target addresses for every call */
  flattened_targets: Fr[];
  /** A nonce for replay protection */
  nonce: Fr;
};

/** Assembles an entrypoint payload from a set of private and public function calls */
function buildPayload(privateCalls: FunctionCall[], publicCalls: FunctionCall[]): EntrypointPayload {
  const nonce = Fr.random();
  const emptyCall = { args: times(ARGS_LENGTH, Fr.zero), selector: Buffer.alloc(32), target: AztecAddress.ZERO };

  const calls = [
    ...padArrayEnd(privateCalls, emptyCall, ACCOUNT_MAX_PRIVATE_CALLS),
    ...padArrayEnd(publicCalls, emptyCall, ACCOUNT_MAX_PUBLIC_CALLS),
  ];

  return {
    // eslint-disable-next-line camelcase
    flattened_args: calls.flatMap(call => padArrayEnd(call.args, Fr.ZERO, ARGS_LENGTH)),
    // eslint-disable-next-line camelcase
    flattened_selectors: calls.map(call => Fr.fromBuffer(call.selector)),
    // eslint-disable-next-line camelcase
    flattened_targets: calls.map(call => call.target.toField()),
    nonce,
  };
}

/** Hashes an entrypoint payload (useful for signing) */
function hashPayload(payload: EntrypointPayload) {
  // TODO: Switch to keccak when avaiable in Noir
  return sha256(Buffer.concat(flattenPayload(payload).map(fr => fr.toBuffer())));
}

/** Flattens an entrypoint payload */
function flattenPayload(payload: EntrypointPayload) {
  return [...payload.flattened_args, ...payload.flattened_selectors, ...payload.flattened_targets, payload.nonce];
}
