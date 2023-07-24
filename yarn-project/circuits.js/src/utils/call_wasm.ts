import { IWasmModule } from '@aztec/foundation/wasm';

import { CircuitError } from '../index.js';
import { uint8ArrayToNum } from './serialize.js';

const CIRCUIT_FAILURE_ERROR_CODE_LENGTH_IN_BYTES = 2;
const CIRCUIT_FAILURE_ERROR_MESSAGE_SIZE_LENGTH_IN_BYTES = 4;

/**
 * Call a wasm method.
 * @param wasm - The wasm module.
 * @param method - The name of the exported wasm method to call.
 * @param input - The input to the wasm method (a buffer or an object serializable to a buffer).
 * @param outputType - The type of the output of the wasm method.
 *
 */
export function callWasm<T>(
  wasm: IWasmModule,
  method: string,
  input:
    | Buffer
    | {
        /**
         * Signature of the target serialization function.
         */
        toBuffer: () => Buffer;
      },
  outputType: {
    /**
     * Signature of the target deserialization function which the output type has to implement.
     */
    fromBuffer: (b: Buffer) => T;
  },
): T {
  const inputBuf: Buffer = input instanceof Buffer ? input : input.toBuffer();

  // Allocate memory for the input buffer and the pointer to the pointer to the output buffer
  const inputBufPtr = wasm.call('bbmalloc', inputBuf.length);
  wasm.writeMemory(inputBufPtr, inputBuf);
  const outputBufSizePtr = wasm.call('bbmalloc', 4);
  const outputBufPtrPtr = wasm.call('bbmalloc', 4);
  // Run and read outputs
  const circuitFailureBufPtr = wasm.call(method, inputBufPtr, outputBufSizePtr, outputBufPtrPtr);

  // Handle wasm output and ensure memory is correctly freed even when an error occurred.
  try {
    const output = handleCircuitOutput(wasm, outputBufSizePtr, outputBufPtrPtr, circuitFailureBufPtr, outputType);
    return output;
  } finally {
    // Free memory
    wasm.call('bbfree', inputBufPtr);
    wasm.call('bbfree', outputBufSizePtr);
    wasm.call('bbfree', outputBufPtrPtr);
    wasm.call('bbfree', circuitFailureBufPtr);
  }
}

/**
 * Tries to deserialize the circuit output into the output type and throws a CircuitError if there was an error.
 * @param wasm - The wasm wrapper.
 * @param outputBufSizePtr - The pointer to the output buffer size.
 * @param outputBufPtrPtr - The pointer to the pointer to the output buffer.
 * @param circuitFailureBufPtr - The pointer to the circuit failure buffer.
 * @param outputType - The type of the output of the wasm method.
 * @returns The deserialized output.
 */
export function handleCircuitOutput<T>(
  wasm: IWasmModule,
  outputBufSizePtr: number,
  outputBufPtrPtr: number,
  circuitFailureBufPtr: number,
  outputType: {
    /**
     * Signature of the target deserialization function which the output type has to implement.
     */
    fromBuffer: (b: Buffer) => T;
  },
): T {
  if (circuitFailureBufPtr != 0) {
    // there is an error: CircuitError struct is structured as:
    // 1st 16 bits (2 bytes) after the `circuitFailureBufPtr` - error code (enum uint16)
    // Next 32 bits (4 bytes) - error message size
    // Next `error message size` bytes - error message.
    // So need to first extract the error message size so we know how much memory to read for the entire error struct.
    const errorMessageSizeBuffer = Buffer.from(
      wasm.getMemorySlice(
        circuitFailureBufPtr + CIRCUIT_FAILURE_ERROR_CODE_LENGTH_IN_BYTES,
        circuitFailureBufPtr +
          CIRCUIT_FAILURE_ERROR_CODE_LENGTH_IN_BYTES +
          CIRCUIT_FAILURE_ERROR_MESSAGE_SIZE_LENGTH_IN_BYTES,
      ),
    );
    const errorMessageSize = errorMessageSizeBuffer.readUint32BE();
    // Now extract the entire `CircuitError` struct:
    const errorBuf = Buffer.from(
      wasm.getMemorySlice(
        circuitFailureBufPtr,
        circuitFailureBufPtr +
          CIRCUIT_FAILURE_ERROR_CODE_LENGTH_IN_BYTES +
          CIRCUIT_FAILURE_ERROR_MESSAGE_SIZE_LENGTH_IN_BYTES +
          errorMessageSize,
      ),
    );
    const err = CircuitError.fromBuffer(errorBuf);
    err.message += '\nRefer to https://docs.aztec.network/aztec/protocol/errors for more information.';
    throw err;
  }
  // C++ returned a null pointer i.e. circuit didn't have an error
  const outputBufSize = uint8ArrayToNum(wasm.getMemorySlice(outputBufSizePtr, outputBufSizePtr + 4));
  const outputBufPtr = uint8ArrayToNum(wasm.getMemorySlice(outputBufPtrPtr, outputBufPtrPtr + 4));
  const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));
  const output = outputType.fromBuffer(outputBuf);
  return output;
}
