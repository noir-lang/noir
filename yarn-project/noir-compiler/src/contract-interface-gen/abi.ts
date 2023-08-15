import { ContractAbi, FunctionAbi, FunctionType } from '@aztec/foundation/abi';

import { mockVerificationKey } from '../mocked_keys.js';
import { NoirCompiledContract, NoirFunctionEntry } from '../noir_artifact.js';

/**
 * Generates an Aztec ABI for a Noir function build artifact. Replaces verification key with a mock value.
 * @param fn - Noir function entry.
 * @returns Aztec ABI function entry.
 */
function generateAbiFunction(fn: NoirFunctionEntry): FunctionAbi {
  const functionType = fn.function_type.toLowerCase() as FunctionType;
  const isInternal = fn.is_internal;

  // If the function is not unconstrained, the first item is inputs or CallContext which we should omit
  let parameters = fn.abi.parameters;
  if (functionType !== FunctionType.UNCONSTRAINED) parameters = parameters.slice(1);

  // If the function is secret, the return is the public inputs, which should be omitted
  const returnTypes = functionType === FunctionType.SECRET ? [] : [fn.abi.return_type];

  return {
    name: fn.name,
    functionType,
    isInternal,
    parameters,
    returnTypes,
    bytecode: fn.bytecode,
    verificationKey: mockVerificationKey,
  };
}

/**
 * Given a Nargo output generates an Aztec-compatible contract ABI.
 * @param compiled - Noir build output.
 * @returns An Aztec valid ABI.
 */
export function generateAztecAbi(compiled: NoirCompiledContract): ContractAbi {
  return {
    name: compiled.name,
    functions: compiled.functions.sort((fnA, fnB) => fnA.name.localeCompare(fnB.name)).map(generateAbiFunction),
  };
}
