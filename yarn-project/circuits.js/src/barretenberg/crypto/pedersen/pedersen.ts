import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';

import { deserializeArrayFromVector, deserializeField, serializeBufferArrayToVector } from '../../serialize.js';

/**
 * Compresses two 32-byte hashes.
 * @param wasm - The barretenberg module.
 * @param lhs - The first hash.
 * @param rhs - The second hash.
 * @returns The new 32-byte hash.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenCompress(wasm: IWasmModule, lhs: Uint8Array, rhs: Uint8Array): Buffer {
  return pedersenCompressWithHashIndex(wasm, [Buffer.from(lhs), Buffer.from(rhs)], 0);
}

/**
 * Combine an array of hashes using pedersen hash.
 * @param wasm - The barretenberg module.
 * @param lhs - The first hash.
 * @param rhs - The second hash.
 * @returns The new 32-byte hash.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenHashInputs(wasm: IWasmModule, inputs: Buffer[]): Buffer {
  return pedersenCompressWithHashIndex(wasm, inputs, 0);
}

/**
 * Compresses an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to compress.
 * @returns The resulting 32-byte hash.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenCompressInputs(wasm: IWasmModule, inputs: Buffer[]): Buffer {
  return pedersenCompressWithHashIndex(wasm, inputs, 0);
}

/**
 * Compresses an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to compress.
 * @param hashIndex - Hash index of the generator to use (See GeneratorIndex enum).
 * @returns The resulting 32-byte hash.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenCompressWithHashIndex(wasm: IWasmModule, inputs: Buffer[], hashIndex: number): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen__compress_with_hash_index', 0, 0, hashIndex);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Given a buffer containing 32 byte pedersen leaves, return a new buffer containing the leaves and all pairs of nodes
 * that define a merkle tree.
 *
 * E.g.
 * Input:  [1][2][3][4]
 * Output: [1][2][3][4][compress(1,2)][compress(3,4)][compress(5,6)].
 *
 * @param wasm - The barretenberg module.
 * @param values - The 32 byte pedersen leaves.
 * @returns A tree represented by an array.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenGetHashTree(wasm: IWasmModule, values: Buffer[]) {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const data = serializeBufferArrayToVector(values);
  const inputPtr = wasm.call('bbmalloc', data.length);
  wasm.writeMemory(inputPtr, data);

  wasm.call('pedersen_hash_to_tree', inputPtr, 0);
  const resultPtr = Buffer.from(wasm.getMemorySlice(0, 4)).readUInt32LE(0);
  // First 4 bytes is full response length in byters.
  // Second 4 bytes is vector length in fields.
  const resultNumFields = Buffer.from(wasm.getMemorySlice(resultPtr + 4, resultPtr + 8)).readUInt32BE(0);
  const resultData = Buffer.from(wasm.getMemorySlice(resultPtr + 4, resultPtr + 8 + resultNumFields * 32));
  wasm.call('bbfree', inputPtr);
  wasm.call('bbfree', resultPtr);

  return deserializeArrayFromVector(deserializeField, resultData).elem;
}
