import { BufferReader } from '@aztec/foundation/serialize';

import { Buffer } from 'buffer';

import {
  FUNCTION_TREE_HEIGHT,
  Fr,
  KernelCircuitPublicInputs,
  PreviousKernelData,
  PrivateCallData,
  TxRequest,
} from '../index.js';
import { handleCircuitOutput } from '../utils/call_wasm.js';
import { serializeBufferArrayToVector, uint8ArrayToNum } from '../utils/serialize.js';
import { CircuitsWasm } from '../wasm/index.js';

export { privateKernelSimOrdering } from '../cbind/circuits.gen.js';

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

/**
 * Computes proof of the private kernel.
 * @param wasm - The circuits wasm instance.
 * @param txRequest - The signed transaction request.
 * @param previousKernel - The previous kernel data (dummy if this is the first kernel in the chain).
 * @param privateCallData - The private call data.
 * @param firstIteration - Whether this is the first iteration of the private kernel.
 * @returns The proof of the private kernel.
 */
export function privateKernelProve(
  wasm: CircuitsWasm,
  txRequest: TxRequest,
  previousKernel: PreviousKernelData,
  privateCallData: PrivateCallData,
  firstIteration: boolean,
): Buffer {
  wasm.call('pedersen__init');
  const txRequestBuffer = txRequest.toBuffer();
  const previousKernelBuffer = previousKernel.toBuffer();
  const privateCallDataBuffer = privateCallData.toBuffer();
  const previousKernelBufferOffset = txRequestBuffer.length;
  const privateCallDataOffset = previousKernelBufferOffset + previousKernelBuffer.length;
  // This is an unused pointer argument at the moment.
  const provingKeyOffset = privateCallDataOffset + privateCallDataBuffer.length;
  wasm.writeMemory(0, txRequestBuffer);
  wasm.writeMemory(previousKernelBufferOffset, previousKernelBuffer);
  wasm.writeMemory(privateCallDataOffset, privateCallDataBuffer);

  const proofOutputAddressPtr = wasm.call('bbmalloc', 4);
  const proofSize = wasm.call(
    'private_kernel__prove',
    0,
    previousKernelBufferOffset,
    privateCallDataOffset,
    provingKeyOffset,
    firstIteration,
    proofOutputAddressPtr,
  );
  // for whenever we actually use this method, we need to do proper error handling in C++ via bberg.
  const address = uint8ArrayToNum(wasm.getMemorySlice(proofOutputAddressPtr, proofOutputAddressPtr + 4));
  const proof = Buffer.from(wasm.getMemorySlice(address, address + proofSize));
  wasm.call('bbfree', proofOutputAddressPtr);
  wasm.call('bbfree', address);
  return proof;
}

/**
 * Computes the public inputs of the private kernel first iteration without computing the proof.
 * @param wasm - The circuits wasm instance.
 * @param txRequest - The signed transaction request.
 * @param privateCallData - The private call data.
 * @returns The public inputs of the private kernel.
 */
export function privateKernelSimInit(
  wasm: CircuitsWasm,
  txRequest: TxRequest,
  privateCallData: PrivateCallData,
): KernelCircuitPublicInputs {
  wasm.call('pedersen__init');
  const txRequestBuffer = txRequest.toBuffer();
  const privateCallDataBuffer = privateCallData.toBuffer();
  const privateCallDataOffset = txRequestBuffer.length;
  wasm.writeMemory(0, txRequestBuffer);
  wasm.writeMemory(privateCallDataOffset, privateCallDataBuffer);
  const outputBufSizePtr = wasm.call('bbmalloc', 4);
  const outputBufPtrPtr = wasm.call('bbmalloc', 4);
  // Run and read outputs
  const circuitFailureBufPtr = wasm.call(
    'private_kernel__sim_init',
    0,
    privateCallDataOffset,
    outputBufSizePtr,
    outputBufPtrPtr,
  );
  try {
    // Try deserializing the output to `KernelCircuitPublicInputs` and throw if it fails
    return handleCircuitOutput(
      wasm,
      outputBufSizePtr,
      outputBufPtrPtr,
      circuitFailureBufPtr,
      KernelCircuitPublicInputs,
    );
  } finally {
    // Free memory
    wasm.call('bbfree', outputBufSizePtr);
    wasm.call('bbfree', outputBufPtrPtr);
    wasm.call('bbfree', circuitFailureBufPtr);
  }
}

/**
 * Computes the public inputs of a private kernel inner iteration without computing the proof.
 * @param wasm - The circuits wasm instance.
 * @param previousKernel - The previous kernel data (dummy if this is the first kernel in the chain).
 * @param privateCallData - The private call data.
 * @returns The public inputs of the private kernel.
 */
export function privateKernelSimInner(
  wasm: CircuitsWasm,
  previousKernel: PreviousKernelData,
  privateCallData: PrivateCallData,
): KernelCircuitPublicInputs {
  wasm.call('pedersen__init');
  const previousKernelBuffer = previousKernel.toBuffer();
  const privateCallDataBuffer = privateCallData.toBuffer();
  const privateCallDataOffset = previousKernelBuffer.length;
  wasm.writeMemory(0, previousKernelBuffer);
  wasm.writeMemory(privateCallDataOffset, privateCallDataBuffer);
  const outputBufSizePtr = wasm.call('bbmalloc', 4);
  const outputBufPtrPtr = wasm.call('bbmalloc', 4);
  // Run and read outputs
  const circuitFailureBufPtr = wasm.call(
    'private_kernel__sim_inner',
    0,
    privateCallDataOffset,
    outputBufSizePtr,
    outputBufPtrPtr,
  );
  try {
    // Try deserializing the output to `KernelCircuitPublicInputs` and throw if it fails
    return handleCircuitOutput(
      wasm,
      outputBufSizePtr,
      outputBufPtrPtr,
      circuitFailureBufPtr,
      KernelCircuitPublicInputs,
    );
  } finally {
    // Free memory
    wasm.call('bbfree', outputBufSizePtr);
    wasm.call('bbfree', outputBufPtrPtr);
    wasm.call('bbfree', circuitFailureBufPtr);
  }
}
