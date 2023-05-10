import { Buffer } from 'buffer';
import {
  Fr,
  FUNCTION_TREE_HEIGHT,
  PreviousKernelData,
  PrivateCallData,
  KernelCircuitPublicInputs,
  SignedTxRequest,
} from '../index.js';
import { boolToBuffer, serializeBufferArrayToVector, uint8ArrayToNum } from '../utils/serialize.js';
import { CircuitsWasm } from '../wasm/index.js';
import { BufferReader } from '@aztec/foundation/serialize';
import { handleCircuitFailure } from '../utils/call_wasm.js';

export async function getDummyPreviousKernelData(wasm: CircuitsWasm) {
  wasm.call('pedersen__init');
  const ptr = wasm.call('bbmalloc', 4);
  const data = await wasm.asyncCall('private_kernel__dummy_previous_kernel', ptr);
  const outputBufSize = uint8ArrayToNum(wasm.getMemorySlice(ptr, ptr + 4));
  wasm.call('bbfree', ptr);
  const result = Buffer.from(wasm.getMemorySlice(data, data + outputBufSize));
  return PreviousKernelData.fromBuffer(result);
}

export async function computeFunctionTree(wasm: CircuitsWasm, leaves: Fr[]): Promise<Fr[]> {
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
  await wasm.asyncCall('abis__compute_function_tree', inputBufPtr, outputBufPtr);
  const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));
  const reader = new BufferReader(outputBuf);
  const output = reader.readVector(Fr);

  // Free memory
  wasm.call('bbfree', outputBufPtr);
  wasm.call('bbfree', inputBufPtr);

  return output;
}

export async function privateKernelProve(
  wasm: CircuitsWasm,
  signedTxRequest: SignedTxRequest,
  previousKernel: PreviousKernelData,
  privateCallData: PrivateCallData,
  firstIteration: boolean,
) {
  wasm.call('pedersen__init');
  const signedTxRequestBuffer = signedTxRequest.toBuffer();
  const previousKernelBuffer = previousKernel.toBuffer();
  const privateCallDataBuffer = privateCallData.toBuffer();
  const previousKernelBufferOffset = signedTxRequestBuffer.length;
  const privateCallDataOffset = previousKernelBufferOffset + previousKernelBuffer.length;
  // The is an unused pointer argument here, so we offset the first iteration arg by 4 further bytes
  const firstInterationOffset = privateCallDataOffset + privateCallDataBuffer.length + 4;
  wasm.writeMemory(0, signedTxRequestBuffer);
  wasm.writeMemory(previousKernelBufferOffset, previousKernelBuffer);
  wasm.writeMemory(privateCallDataOffset, privateCallDataBuffer);
  wasm.writeMemory(firstInterationOffset, boolToBuffer(firstIteration));

  const proofOutputAddressPtr = wasm.call('bbmalloc', 4);
  const proofSize = await wasm.asyncCall(
    'private_kernel__prove',
    0,
    previousKernelBufferOffset,
    privateCallDataOffset,
    firstInterationOffset,
    firstInterationOffset,
    proofOutputAddressPtr,
  );
  // for whenever we actually use this method, we need to do proper error handling in C++ via the bberg composer.
  const address = uint8ArrayToNum(wasm.getMemorySlice(proofOutputAddressPtr, proofOutputAddressPtr + 4));
  const proof = Buffer.from(wasm.getMemorySlice(address, address + proofSize));
  wasm.call('bbfree', proofOutputAddressPtr);
  wasm.call('bbfree', address);
  return proof;
}

export async function privateKernelSim(
  wasm: CircuitsWasm,
  signedTxRequest: SignedTxRequest,
  previousKernel: PreviousKernelData,
  privateCallData: PrivateCallData,
  firstIteration: boolean,
) {
  wasm.call('pedersen__init');
  const signedTxRequestBuffer = signedTxRequest.toBuffer();
  const previousKernelBuffer = previousKernel.toBuffer();
  const privateCallDataBuffer = privateCallData.toBuffer();
  const previousKernelBufferOffset = signedTxRequestBuffer.length;
  const privateCallDataOffset = previousKernelBufferOffset + previousKernelBuffer.length;
  const firstInterationOffset = privateCallDataOffset + privateCallDataBuffer.length;
  wasm.writeMemory(0, signedTxRequestBuffer);
  wasm.writeMemory(previousKernelBufferOffset, previousKernelBuffer);
  wasm.writeMemory(privateCallDataOffset, privateCallDataBuffer);
  wasm.writeMemory(firstInterationOffset, boolToBuffer(firstIteration));
  const outputBufSizePtr = wasm.call('bbmalloc', 4);
  const outputBufPtrPtr = wasm.call('bbmalloc', 4);
  // Run and read outputs
  const circuitFailureBufPtr = await wasm.asyncCall(
    'private_kernel__sim',
    0,
    previousKernelBufferOffset,
    privateCallDataOffset,
    firstInterationOffset,
    outputBufSizePtr,
    outputBufPtrPtr,
  );
  try {
    return handleCircuitFailure(
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
