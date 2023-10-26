import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';

import { deserializeArrayFromVector, deserializeField, serializeBufferArrayToVector } from '../../serialize.js';

/**
 * Hashes two arrays.
 * @param wasm - The barretenberg module.
 * @param lhs - The first array.
 * @param rhs - The second array.
 * @returns The new 32-byte hash.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenHash(wasm: IWasmModule, lhs: Uint8Array, rhs: Uint8Array): Buffer {
  return pedersenHashWithHashIndex(wasm, [Buffer.from(lhs), Buffer.from(rhs)], 0);
}

/**
 * Computes the hash of an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to hash.
 * @returns The new 32-byte hash.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenHashInputs(wasm: IWasmModule, inputs: Buffer[]): Buffer {
  return pedersenHashWithHashIndex(wasm, inputs, 0);
}

/**
 * Hashes an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to hash.
 * @param hashIndex - Hash index of the generator to use (See GeneratorIndex enum).
 * @returns The resulting 32-byte hash.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export function pedersenHashWithHashIndex(wasm: IWasmModule, inputs: Buffer[], hashIndex: number): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');

  const data = serializeBufferArrayToVector(inputs);

  // WASM gives us 1024 bytes of scratch space which we can use without
  // needing to allocate/free it ourselves. This can be useful for when we need to pass in several small variables
  // when calling functions on the wasm, however it's important to not overrun this scratch space as otherwise
  // the written data will begin to corrupt the stack.
  //
  // Using this scratch space isn't particularly safe if we have multiple threads interacting with the wasm however,
  // each thread could write to the same pointer address simultaneously.
  const SCRATCH_SPACE_SIZE = 1024;

  // For pedersen hashing, the case of hashing two inputs is the most common.
  // so ideally we want to optimize for that. This will use 64 bytes of memory and
  // can thus be optimized by checking if the input buffer is smaller than the scratch space.
  let inputPtr = 0;
  if (inputs.length >= SCRATCH_SPACE_SIZE) {
    inputPtr = wasm.call('bbmalloc', data.length);
  }
  wasm.writeMemory(inputPtr, data);

  // Since the output is 32 bytes, instead of allocating memory
  // we can reuse the scratch space to store the result.
  const outputPtr = 0;

  wasm.call('pedersen__compress_with_hash_index', inputPtr, hashIndex, outputPtr);
  const hashOutput = wasm.getMemorySlice(0, 32);

  wasm.call('bbfree', inputPtr);

  return Buffer.from(hashOutput);
}

/**
 * Given a buffer containing 32 byte pedersen leaves, return a new buffer containing the leaves and all pairs of nodes
 * that define a merkle tree.
 *
 * E.g.
 * Input:  [1][2][3][4]
 * Output: [1][2][3][4][hash(1,2)][hash(3,4)][hash(5,6)].
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
