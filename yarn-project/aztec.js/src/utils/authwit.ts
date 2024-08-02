import { type FunctionCall, PackedValues } from '@aztec/circuit-types';
import { type AztecAddress, Fr, GeneratorIndex } from '@aztec/circuits.js';
import { poseidon2HashWithSeparator } from '@aztec/foundation/crypto';

import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';

/** Metadata for the intent */
export type IntentMetadata = {
  /** The chain id to approve */
  chainId: Fr;
  /** The version to approve  */
  version: Fr;
};

/** Intent with an inner hash */
export type IntentInnerHash = {
  /** The consumer   */
  consumer: AztecAddress;
  /** The action to approve */
  innerHash: Buffer | Fr;
};

/** Intent with an action */
export type IntentAction = {
  /** The caller to approve  */
  caller: AztecAddress;
  /** The action to approve */
  action: ContractFunctionInteraction | FunctionCall;
};

// docs:start:authwit_computeAuthWitMessageHash
/**
 * Compute an authentication witness message hash from an intent and metadata
 *
 * If using the `IntentInnerHash`, the consumer is the address that can "consume" the authwit, for token approvals it is the token contract itself.
 * The `innerHash` itself will be the message that a contract is allowed to execute.
 * At the point of "approval checking", the validating contract (account for private and registry for public) will be computing the message hash
 * (`H(consumer, chainid, version, inner_hash)`) where the all but the `inner_hash` is injected from the context (consumer = msg_sender),
 * and use it for the authentication check.
 * Therefore, any allowed `innerHash` will therefore also have information around where it can be spent (version, chainId) and who can spend it (consumer).
 *
 * If using the `IntentAction`, the caller is the address that is making the call, for a token approval from Alice to Bob, this would be Bob.
 * The action is then used along with the `caller` to compute the `innerHash` and the consumer.
 *
 *
 * @param intent - The intent to approve (consumer and innerHash or caller and action)
 *                 The consumer is the address that can "consume" the authwit, for token approvals it is the token contract itself.
 *                 The caller is the address that is making the call, for a token approval from Alice to Bob, this would be Bob.
 *                 The caller becomes part of the `inner_hash` and is dealt with entirely in application logic.
 * @param metadata - The metadata for the intent (chainId, version)
 * @returns The message hash for the action
 */
export const computeAuthWitMessageHash = (intent: IntentInnerHash | IntentAction, metadata: IntentMetadata) => {
  const chainId = metadata.chainId;
  const version = metadata.version;

  if ('caller' in intent) {
    const action = intent.action instanceof ContractFunctionInteraction ? intent.action.request() : intent.action;
    return computeOuterAuthWitHash(
      action.to.toField(),
      chainId,
      version,
      computeInnerAuthWitHashFromAction(intent.caller, action),
    );
  } else {
    const inner = Buffer.isBuffer(intent.innerHash) ? Fr.fromBuffer(intent.innerHash) : intent.innerHash;
    return computeOuterAuthWitHash(intent.consumer, chainId, version, inner);
  }
};
// docs:end:authwit_computeAuthWitMessageHash

export const computeInnerAuthWitHashFromAction = (caller: AztecAddress, action: FunctionCall) =>
  computeInnerAuthWitHash([caller.toField(), action.selector.toField(), PackedValues.fromValues(action.args).hash]);

/**
 * Compute the inner hash for an authentication witness.
 * This is the "intent" of the message, before siloed with the consumer.
 * It is used as part of the `computeAuthWitMessageHash` but can also be used
 * in case the message is not a "call" to a function, but arbitrary data.
 * @param args - The arguments to hash
 * @returns The inner hash for the witness
 */
export const computeInnerAuthWitHash = (args: Fr[]) => {
  return poseidon2HashWithSeparator(args, GeneratorIndex.AUTHWIT_INNER);
};

/**
 * Compute the outer hash for an authentication witness.
 * This is the value siloed with its "consumer" and what the `on_behalf_of`
 * should be signing.
 * The consumer is who will be consuming the message, for token approvals it
 * is the token contract itself (because the token makes the call to check the approval).
 * It is used as part of the `computeAuthWitMessageHash` but can also be used
 * in case the message is not a "call" to a function, but arbitrary data.
 * @param consumer - The address that can "consume" the authwit
 * @param chainId - The chain id that can "consume" the authwit
 * @param version - The version that can "consume" the authwit
 * @param innerHash - The inner hash for the witness
 * @returns The outer hash for the witness
 */
const computeOuterAuthWitHash = (consumer: AztecAddress, chainId: Fr, version: Fr, innerHash: Fr) => {
  return poseidon2HashWithSeparator([consumer.toField(), chainId, version, innerHash], GeneratorIndex.AUTHWIT_OUTER);
};
