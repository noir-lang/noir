import { FunctionCall, PackedArguments } from '@aztec/circuit-types';
import { AztecAddress, Fr, GeneratorIndex } from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';

// docs:start:authwit_computeAuthWitMessageHash
/**
 * Compute an authentication witness message hash from a caller and a request
 * H(target: AztecAddress, H(caller: AztecAddress, selector: Field, args_hash: Field))
 * Example usage would be `bob` authenticating `alice` to perform a transfer of `10`
 * tokens from his account to herself:
 * H(token, H(alice, transfer_selector, H(bob, alice, 10, nonce)))
 * `bob` then signs the message hash and gives it to `alice` who can then perform the
 * action.
 * @param caller - The caller approved to make the call
 * @param request - The request to be made (function call)
 * @returns The message hash for the witness
 */
export const computeAuthWitMessageHash = (caller: AztecAddress, request: FunctionCall) => {
  return computeOuterAuthWitHash(
    request.to.toField(),
    computeInnerAuthWitHash([
      caller.toField(),
      request.functionData.selector.toField(),
      PackedArguments.fromArgs(request.args).hash,
    ]),
  );
};
// docs:end:authwit_computeAuthWitMessageHash

/**
 * Compute the inner hash for an authentication witness.
 * This is the "intent" of the message, before siloed with the consumer.
 * It is used as part of the `computeAuthWitMessageHash` but can also be used
 * in case the message is not a "call" to a function, but arbitrary data.
 * @param args - The arguments to hash
 * @returns The inner hash for the witness
 */
export const computeInnerAuthWitHash = (args: Fr[]) => {
  return pedersenHash(
    args.map(fr => fr.toBuffer()),
    GeneratorIndex.AUTHWIT_INNER,
  );
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
 * @param innerHash - The inner hash for the witness
 * @returns The outer hash for the witness
 */
export const computeOuterAuthWitHash = (consumer: AztecAddress, innerHash: Fr) => {
  return pedersenHash(
    [consumer.toField(), innerHash].map(fr => fr.toBuffer()),
    GeneratorIndex.AUTHWIT_OUTER,
  );
};
