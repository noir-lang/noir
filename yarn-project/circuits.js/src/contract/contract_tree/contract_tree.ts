import { CircuitsWasm, ContractFunctionDao, Fr, FunctionData, FunctionLeafPreimage } from '@aztec/circuits.js';
import { computeFunctionLeaf, hashVK } from '@aztec/circuits.js/abis';
import { FunctionType, generateFunctionSelector } from '@aztec/foundation/abi';

/**
 * Computes the hash of a hex-encoded string representation of a verification key (vk).
 * The input 'vk' should be a hexadecimal string, and the resulting hash is computed using 'hashVK' function.
 * Returns a Promise that resolves to a Buffer containing the hash of the verification key.
 *
 * @param vk - The hex-encoded string representing the verification key.
 * @param wasm - An instance of CircuitsWasm class used for hashing.
 * @returns A Promise resolving to a Buffer containing the hash of the verification key.
 */
export function hashVKStr(vk: string, wasm: CircuitsWasm) {
  // TODO - check consistent encoding
  return hashVK(wasm, Buffer.from(vk, 'hex'));
}

/**
 * Determine if the given function is a constructor.
 * This utility function checks if the 'name' property of the input object is "constructor".
 * Returns true if the function is a constructor, false otherwise.
 *
 * @param Object - An object containing a 'name' property.
 * @returns Boolean indicating if the function is a constructor.
 */
export function isConstructor({
  name,
}: {
  /**
   * Function name identifier.
   */
  name: string;
}) {
  return name === 'constructor';
}

/**
 * @param Object - An object containing function name and type.
 * @returns Boolean indicating if the function is constrained and therefore in the function tree.
 */
export function isConstrained({
  name,
  functionType,
}: {
  /**
   * The name of the contract function.
   */
  name: string;
  /**
   * The type of a contract function determining its constraints.
   */
  functionType: FunctionType;
}) {
  return functionType !== FunctionType.UNCONSTRAINED && !isConstructor({ name });
}

/**
 * Generate function leaves for the constrained functions in a contract.
 * Only computes leaves for functions that are either secret or open and not constructors.
 * Each function leaf is computed from its selector, privacy flag, hashed verification key, and hashed bytecode.
 *
 * @param functions - Array of ContractFunctionDao objects representing the functions in a contract.
 * @param wasm - CircuitsWasm instance used for hashing and computations.
 * @returns An array of Fr instances representing the generated function leaves.
 */
export function generateFunctionLeaves(functions: ContractFunctionDao[], wasm: CircuitsWasm) {
  const targetFunctions = functions.filter(isConstrained);
  const result: Fr[] = [];
  for (let i = 0; i < targetFunctions.length; i++) {
    const f = targetFunctions[i];
    const selector = generateFunctionSelector(f.name, f.parameters);
    const isInternal = f.isInternal;
    const isPrivate = f.functionType === FunctionType.SECRET;
    // All non-unconstrained functions have vks
    const vkHash = hashVKStr(f.verificationKey!, wasm);
    // TODO
    // FIXME: https://github.com/AztecProtocol/aztec3-packages/issues/262
    // const acirHash = keccak(Buffer.from(f.bytecode, 'hex'));
    const acirHash = Buffer.alloc(32, 0);

    const fnLeafPreimage = new FunctionLeafPreimage(
      selector,
      isInternal,
      isPrivate,
      Fr.fromBuffer(vkHash),
      Fr.fromBuffer(acirHash),
    );
    const fnLeaf = computeFunctionLeaf(wasm, fnLeafPreimage);
    result.push(fnLeaf);
  }
  return result;
}

/**
 * Represents the constructor data for a new contract.
 * Contains the function data and verification key hash required for contract creation.
 */
export interface NewContractConstructor {
  /**
   * Stores essential information about a contract function.
   */
  functionData: FunctionData;
  /**
   * The hashed verification key of a function.
   */
  vkHash: Buffer;
}
