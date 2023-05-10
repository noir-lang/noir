import { CircuitError } from '../index.js';
import { AsyncWasmWrapper } from '@aztec/foundation/wasm';
import { uint8ArrayToNum } from './serialize.js';

const CIRCUIT_FAILURE_ERROR_CODE_LENGTH_IN_BYTES = 2;
const CIRCUIT_FAILURE_ERROR_MESSAGE_SIZE_LENGTH_IN_BYTES = 4;

export async function callAsyncWasm<T>(
  wasm: AsyncWasmWrapper,
  method: string,
  input: Buffer | { toBuffer: () => Buffer },
  outputType: { fromBuffer: (b: Buffer) => T },
): Promise<T> {
  const inputBuf: Buffer = input instanceof Buffer ? input : input.toBuffer();

  // Allocate memory for the input buffer and the pointer to the pointer to the output buffer
  const inputBufPtr = wasm.call('bbmalloc', inputBuf.length);
  wasm.writeMemory(inputBufPtr, inputBuf);
  const outputBufSizePtr = wasm.call('bbmalloc', 4);
  const outputBufPtrPtr = wasm.call('bbmalloc', 4);
  // Run and read outputs
  const circuitFailureBufPtr = await wasm.asyncCall(method, inputBufPtr, outputBufSizePtr, outputBufPtrPtr);

  // handle circuit failure but in either case, free the input buffer from memory.
  try {
    const output = handleCircuitFailure(wasm, outputBufSizePtr, outputBufPtrPtr, circuitFailureBufPtr, outputType);
    return output;
  } finally {
    // Free memory
    wasm.call('bbfree', inputBufPtr);
    wasm.call('bbfree', outputBufSizePtr);
    wasm.call('bbfree', outputBufPtrPtr);
    wasm.call('bbfree', circuitFailureBufPtr);
  }
}

export function handleCircuitFailure<T>(
  wasm: AsyncWasmWrapper,
  outputBufSizePtr: number,
  outputBufPtrPtr: number,
  circuitFailureBufPtr: number,
  outputType: { fromBuffer: (b: Buffer) => T },
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
    throw err;
  }
  // C++ returned a null pointer i.e. circuit didn't have an error
  const outputBufSize = uint8ArrayToNum(wasm.getMemorySlice(outputBufSizePtr, outputBufSizePtr + 4));
  const outputBufPtr = uint8ArrayToNum(wasm.getMemorySlice(outputBufPtrPtr, outputBufPtrPtr + 4));
  const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));
  const output = outputType.fromBuffer(outputBuf);
  return output;
}
