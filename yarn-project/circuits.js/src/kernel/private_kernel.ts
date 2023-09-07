import { BufferReader } from '@aztec/foundation/serialize';

import { Buffer } from 'buffer';

import { FUNCTION_TREE_HEIGHT, Fr } from '../index.js';
import { serializeBufferArrayToVector } from '../utils/serialize.js';
import { CircuitsWasm } from '../wasm/index.js';

export { privateKernelSimOrdering, privateKernelSimInit, privateKernelSimInner } from '../cbind/circuits.gen.js';

/**
 * Computes contract's function tree from the given leaves.
 * @param wasm - The circuits wasm instance.
 * @param leaves - The leaves of the function tree.
 * @returns All of a function tree's nodes.
 */
export function computeFunctionTree(wasm: CircuitsWasm, leaves: Fr[]): Fr[] {
  // Init pedersen if needed
  wasm.call('pedersen__init');

  // Size of the tree is 2^height times size of each element,
  // plus 4 for the size used in the std::vector serialization
  const outputBufSize = 2 ** (FUNCTION_TREE_HEIGHT + 1) * Fr.SIZE_IN_BYTES + 4;

  // Allocate memory for the input and output buffers, and populate input buffer
  const inputVector = serializeBufferArrayToVector(leaves.map(fr => fr.toBuffer()));
  const inputBufPtr = wasm.call('bbmalloc', inputVector.length);
  const outputBufPtr = wasm.call('bbmalloc', outputBufSize * 100);
  wasm.writeMemory(inputBufPtr, inputVector);

  // Run and read outputs
  wasm.call('abis__compute_function_tree', inputBufPtr, outputBufPtr);
  const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));
  const reader = new BufferReader(outputBuf);
  const output = reader.readVector(Fr);

  // Free memory
  wasm.call('bbfree', outputBufPtr);
  wasm.call('bbfree', inputBufPtr);

  return output;
}
