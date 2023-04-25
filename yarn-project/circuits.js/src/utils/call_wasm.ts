import { AsyncWasmWrapper } from '@aztec/foundation/wasm';
import { uint8ArrayToNum } from './serialize.js';

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
  const outputBufPtrPtr = wasm.call('bbmalloc', 4);

  // Run and read outputs
  const outputBufSize = await wasm.asyncCall(method, inputBufPtr, outputBufPtrPtr);
  const outputBufPtr = uint8ArrayToNum(wasm.getMemorySlice(outputBufPtrPtr, outputBufPtrPtr + 4));
  const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));
  const output = outputType.fromBuffer(outputBuf);

  // Free memory
  wasm.call('bbfree', outputBufPtr);
  wasm.call('bbfree', outputBufPtrPtr);
  wasm.call('bbfree', inputBufPtr);

  return output;
}
