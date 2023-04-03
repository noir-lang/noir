import { BarretenbergWasm } from '../../wasm/index.js';
import { deserializeArrayFromVector, deserializeField, serializeBufferArrayToVector } from '../../wasm/serialize.js';
import { Buffer } from 'buffer';

/**
 * Combines two 32-byte hashes.
 * @param wasm - The barretenberg module.
 * @param lhs - The first hash.
 * @param rhs - The second hash.
 * @returns The new 32-byte hash.
 */
export function pedersenCompress(wasm: BarretenbergWasm, lhs: Uint8Array, rhs: Uint8Array): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  
  if (lhs.length > 32 || rhs.length > 32) {
    throw new Error('lhs and rhs must not be greater than 32 bytes')
  }
  
  wasm.writeMemory(0, lhs);
  wasm.writeMemory(32, rhs);
  wasm.call('pedersen__hash_pair', 0, 32, 64);
  return Buffer.from(wasm.getMemorySlice(64, 96));
}

/**
 * Combine an array of hashes.
 * @param wasm - The barretenberg module.
 * @param lhs - The first hash.
 * @param rhs - The second hash.
 * @returns The new 32-byte hash.
 */
export function pedersenCompressInputs(wasm: BarretenbergWasm, inputs: Buffer[]): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen__compress', 0, 0);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Combine an array of hashes.
 * @param wasm - The barretenberg module.
 * @param lhs - The first hash.
 * @param rhs - The second hash.
 * @returns The new 32-byte hash.
 */
export function pedersenCompressWithHashIndex(wasm: BarretenbergWasm, inputs: Buffer[], hashIndex: number): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen__compress_with_hash_index', 0, 0, hashIndex);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Get a 32-byte pedersen hash from a buffer.
 * @param wasm - The barretenberg module.
 * @param data - The data buffer.
 * @returns The hash buffer.
 */
export function pedersenGetHash(wasm: BarretenbergWasm, data: Buffer): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const mem = wasm.call('bbmalloc', data.length);
  wasm.writeMemory(mem, data);
  wasm.call('pedersen__buffer_to_field', mem, data.length, 0);
  wasm.call('bbfree', mem);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Given a buffer containing 32 byte pedersen leaves, return a new buffer containing the leaves and all pairs of nodes that define a merkle tree.
 * E.g.
 * Input:  [1][2][3][4]
 * Output: [1][2][3][4][compress(1,2)][compress(3,4)][compress(5,6)].
 * @param wasm - The barretenberg module.
 * @param values - The 32 byte pedersen leaves.
 * @returns A tree represented by an array.
 */
export function pedersenGetHashTree(wasm: BarretenbergWasm, values: Buffer[]) {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const data = serializeBufferArrayToVector(values);
  const inputPtr = wasm.call('bbmalloc', data.length);
  wasm.writeMemory(inputPtr, data);

  const resultPtr = wasm.call('pedersen__hash_to_tree', inputPtr);
  const resultNumFields = Buffer.from(wasm.getMemorySlice(resultPtr, resultPtr + 4)).readUInt32BE(0);
  const resultData = Buffer.from(wasm.getMemorySlice(resultPtr, resultPtr + 4 + resultNumFields * 32));
  wasm.call('bbfree', inputPtr);
  wasm.call('bbfree', resultPtr);

  return deserializeArrayFromVector(deserializeField, resultData).elem;
}
