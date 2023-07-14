import { IWasmModule } from '@aztec/foundation/wasm';

import { decode, encode } from '@msgpack/msgpack';

import { CircuitsWasm } from '../wasm/index.js';

/**
 * Recursively converts Uint8Arrays to Buffers in the input data structure.
 * The function traverses through the given data, and if it encounters a Uint8Array,
 * it replaces it with a Buffer. It supports nested arrays and objects.
 *
 * @param data - The input data structure that may contain Uint8Arrays.
 * @returns A new data structure with all instances of Uint8Array replaced by Buffer.
 */
function recursiveUint8ArrayToBuffer(data: any): any {
  if (Array.isArray(data)) {
    return data.map(recursiveUint8ArrayToBuffer);
  } else if (data instanceof Uint8Array) {
    return Buffer.from(data);
  } else if (data && typeof data === 'object') {
    const fixed: any = {};

    for (const key in data) {
      fixed[key] = recursiveUint8ArrayToBuffer(data[key]);
    }

    return fixed;
  } else {
    return data;
  }
}

/**
 * Read a 32-bit pointer value from the WebAssembly memory space.
 *
 * @param wasm - The CircuitsWasm.
 * @param ptr32 - The address in WebAssembly memory.
 * @returns The read unsigned 32-bit integer.
 */
function readPtr32(wasm: IWasmModule, ptr32: number) {
  // Written in little-endian as WASM native
  const dataView = new DataView(wasm.getMemorySlice(ptr32, ptr32 + 4).buffer);
  return dataView.getUint32(0, /*little endian*/ true);
}

/**
 * Retrieves the JSON schema of a given C binding function from the WebAssembly module.
 *
 * @param wasm - The CircuitsWasm.
 * @param cbind - The name of the function.
 * @returns A JSON object representing the schema.
 */
export function getCbindSchema(wasm: CircuitsWasm, cbind: string): any {
  const outputSizePtr = wasm.call('bbmalloc', 4);
  const outputMsgpackPtr = wasm.call('bbmalloc', 4);
  wasm.call(cbind + '__schema', outputMsgpackPtr, outputSizePtr);
  const jsonSchema = wasm.getMemoryAsString(readPtr32(wasm, outputMsgpackPtr));
  wasm.call('bbfree', outputSizePtr);
  wasm.call('bbfree', outputMsgpackPtr);
  return JSON.parse(jsonSchema);
}

/**
 * Calls a C binding function in the WebAssembly module with the provided input arguments.
 *
 * @param wasm - The CircuitsWasm.
 * @param cbind - The name of function.
 * @param input - An array of input arguments to wrap with msgpack.
 * @returns The msgpack-decoded result.
 */
export function callCbind(wasm: IWasmModule, cbind: string, input: any[]): any {
  const outputSizePtr = wasm.call('bbmalloc', 4);
  const outputMsgpackPtr = wasm.call('bbmalloc', 4);
  const inputBuffer = encode(input);
  const inputPtr = wasm.call('bbmalloc', inputBuffer.length);
  wasm.writeMemory(inputPtr, inputBuffer);
  wasm.call(cbind, inputPtr, inputBuffer.length, outputMsgpackPtr, outputSizePtr);
  const encodedResult = wasm.getMemorySlice(
    readPtr32(wasm, outputMsgpackPtr),
    readPtr32(wasm, outputMsgpackPtr) + readPtr32(wasm, outputSizePtr),
  );
  const result = recursiveUint8ArrayToBuffer(decode(encodedResult));
  wasm.call('bbfree', inputPtr);
  wasm.call('bbfree', outputSizePtr);
  wasm.call('bbfree', outputMsgpackPtr);
  return result;
}
