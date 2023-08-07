import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';

import { deserializeArrayFromVector, deserializeField, serializeBufferArrayToVector } from '../../serialize.js';

/**
 * Compresses two 32-byte hashes.
 * @param wasm - The barretenberg module.
 * @param lhs - The first hash.
 * @param rhs - The second hash.
 * @returns The new 32-byte hash.
 */
export function pedersenCompress(wasm: IWasmModule, lhs: Uint8Array, rhs: Uint8Array): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  if (lhs.length !== 32 || rhs.length !== 32) {
    throw new Error(`Pedersen lhs and rhs inputs must be 32 bytes (got ${lhs.length} and ${rhs.length} respectively)`);
  }
  wasm.writeMemory(0, lhs);
  wasm.writeMemory(32, rhs);
  wasm.call('pedersen__hash_pair', 0, 32, 64);
  return Buffer.from(wasm.getMemorySlice(64, 96));
}

/**
 * Combine an array of hashes using pedersen hash.
 * @param wasm - The barretenberg module.
 * @param lhs - The first hash.
 * @param rhs - The second hash.
 * @returns The new 32-byte hash.
 */
export function pedersenHashInputs(wasm: IWasmModule, inputs: Buffer[]): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen__hash_multiple', 0, 0);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Compresses an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to compress.
 * @returns The resulting 32-byte hash.
 */
export function pedersenCompressInputs(wasm: IWasmModule, inputs: Buffer[]): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen__compress', 0, 0);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Commits to an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to compress.
 * @returns The resulting 32-byte hash.
 */
export function pedersenPlookupCommitInputs(wasm: IWasmModule, inputs: Buffer[]): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen_plookup_commit', 0, 0);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Commits an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to compress.
 * @param hashIndex - Hash index of the generator to use (See GeneratorIndex enum).
 * @returns The resulting 32-byte hash.
 */
export function pedersenPlookupCommitWithHashIndex(wasm: IWasmModule, inputs: Buffer[], hashIndex: number): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen_plookup_commit_with_hash_index', 0, 0, hashIndex);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Compresses an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to compress.
 * @param hashIndex - Hash index of the generator to use (See GeneratorIndex enum).
 * @returns The resulting 32-byte hash.
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
 * Compresses an array of buffers.
 * @param wasm - The barretenberg module.
 * @param inputs - The array of buffers to compress.
 * @param hashIndex - Hash index of the generator to use (See GeneratorIndex enum).
 * @returns The resulting 32-byte hash.
 */
export function pedersenPlookupCompressWithHashIndex(wasm: IWasmModule, inputs: Buffer[], hashIndex: number): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const inputVectors = serializeBufferArrayToVector(inputs);
  wasm.writeMemory(0, inputVectors);
  wasm.call('pedersen_plookup_compress_with_hash_index', 0, 0, hashIndex);
  return Buffer.from(wasm.getMemorySlice(0, 32));
}

/**
 * Get a 32-byte pedersen hash from a buffer.
 * @param wasm - The barretenberg module.
 * @param data - The data buffer.
 * @returns The hash buffer.
 */
export function pedersenGetHash(wasm: IWasmModule, data: Buffer): Buffer {
  // If not done already, precompute constants.
  wasm.call('pedersen__init');
  const mem = wasm.call('bbmalloc', data.length);
  wasm.writeMemory(mem, data);
  wasm.call('pedersen__buffer_to_field', mem, data.length, 0);
  wasm.call('bbfree', mem);
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
 */
export function pedersenGetHashTree(wasm: IWasmModule, values: Buffer[]) {
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
