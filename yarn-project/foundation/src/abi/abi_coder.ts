import { ABIParameter, ABIType } from '@aztec/foundation/abi';
import { keccak } from '@aztec/foundation/crypto';

/**
 * Generate a function signature string for a given function name and parameters.
 * The signature is used to uniquely identify functions within noir contracts.
 * If the function name is 'constructor', it returns just the name, otherwise it returns the name followed by the list of parameter types.
 *
 * @param name - The name of the function.
 * @param parameters - An array of ABIParameter objects, each containing the type information of a function parameter.
 * @returns A string representing the function signature.
 */
export function computeFunctionSignature(name: string, parameters: ABIParameter[]) {
  return name === 'constructor' ? name : `${name}(${parameters.map(p => p.type.kind).join(',')})`;
}

/**
 * Generate a function selector for a given function signature.
 * @param signature - The signature of the function.
 * @param size - Number of bytes of the return buffer.
 * @returns A Buffer containing the n-byte function selector.
 */
export function computeFunctionSelector(signature: string, size: number) {
  return keccak(Buffer.from(signature)).slice(0, size);
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
  const signature = computeFunctionSignature(name, parameters);
  return keccak(Buffer.from(signature)).slice(0, 4);
}

/**
 * Get the size of an ABI type in field elements.
 * @param type - The ABI type.
 * @returns The size of the type in field elements.
 */
export function sizeOfType(type: ABIType): number {
  switch (type.kind) {
    case 'field':
    case 'boolean':
    case 'integer':
      return 1;
    case 'string':
      return type.length;
    case 'array':
      return type.length * sizeOfType(type.type);
    case 'struct':
      return type.fields.reduce((sum, field) => sum + sizeOfType(field.type), 0);
    default: {
      const exhaustiveCheck: never = type;
      throw new Error(`Unhandled abi type: ${exhaustiveCheck}`);
    }
  }
}
