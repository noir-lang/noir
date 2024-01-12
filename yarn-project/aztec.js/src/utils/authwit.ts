import { FunctionCall, PackedArguments } from '@aztec/circuit-types';
import { AztecAddress, GeneratorIndex } from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';

// docs:start:authwit_computeAuthWitMessageHash
/**
 * Compute an authentication witness message hash from a caller and a request
 * H(caller: AztecAddress, target: AztecAddress, selector: Field, args_hash: Field)
 * @param caller - The caller approved to make the call
 * @param request - The request to be made (function call)
 * @returns The message hash for the witness
 */
export const computeAuthWitMessageHash = (caller: AztecAddress, request: FunctionCall) => {
  return pedersenHash(
    [
      caller.toField(),
      request.to.toField(),
      request.functionData.selector.toField(),
      PackedArguments.fromArgs(request.args).hash,
    ].map(fr => fr.toBuffer()),
    GeneratorIndex.SIGNATURE_PAYLOAD,
  );
};
// docs:end:authwit_computeAuthWitMessageHash
