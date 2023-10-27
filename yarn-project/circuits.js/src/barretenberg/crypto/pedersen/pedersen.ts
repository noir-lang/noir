import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';

import { serializeBufferArrayToVector } from '../../serialize.js';

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

  wasm.call('pedersen__hash_with_hash_index', inputPtr, hashIndex, outputPtr);
  const hashOutput = wasm.getMemorySlice(0, 32);

  wasm.call('bbfree', inputPtr);

  return Buffer.from(hashOutput);
}
