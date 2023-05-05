import { keccak } from '@aztec/foundation/crypto';
import { ABIParameter } from '@aztec/noir-contracts';

/**
 * Generate a function signature string for a given function name and parameters.
 * The signature is used to uniquely identify functions within noir contracts.
 * If the function name is 'constructor', it returns just the name, otherwise it returns the name followed by the list of parameter types.
 *
 * @param name - The name of the function.
 * @param parameters - An array of ABIParameter objects, each containing the type information of a function parameter.
 * @returns A string representing the function signature.
 */
export function generateFunctionSignature(name: string, parameters: ABIParameter[]) {
  return name === 'constructor' ? name : `${name}(${parameters.map(p => p.type.kind).join(',')})`;
}

/**
 * Generate a function selector for a given function name and parameters.
 * It is derived by taking the first 4 bytes of the Keccak-256 hash of the function signature.
 *
 * @param name - The name of the function.
 * @param parameters - An array of ABIParameter objects, each containing the type information of a function parameter.
 * @returns A Buffer containing the 4-byte function selector.
 */
export function generateFunctionSelector(name: string, parameters: ABIParameter[]) {
  const signature = generateFunctionSignature(name, parameters);
  return keccak(Buffer.from(signature)).slice(0, 4);
}
