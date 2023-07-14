import { uint8ArrayToNum } from '../utils/serialize.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

/**
 * Simplify e.g. 0x0003 into 0x3.
 * @param input - The input string, with hex somewhere inside.
 * @returns The output string with fixed hex.
 */
export function simplifyHexValues(input: string) {
  const regex = /0x[\dA-Fa-f]+/g;
  const matches = input.match(regex) || [];
  const simplifiedMatches = matches.map(match => '0x' + BigInt(match).toString(16));
  const result = input.replace(regex, () => simplifiedMatches.shift() || '');
  return result;
}

/**
 * Test utility. Sends a serialized buffer to wasm and gets the result.
 * @param inputBuf - Buffer to write.
 * @param serializeMethod - Method to use buffer with.
 * @param wasm - Optional circuit wasm. If not set, we fetch a singleton.
 */
async function callWasm(inputBuf: Buffer, serializeMethod: string, wasm?: CircuitsWasm): Promise<Buffer> {
  wasm = wasm || (await CircuitsWasm.get());
  const inputBufPtr = wasm.call('bbmalloc', inputBuf.length);
  wasm.writeMemory(inputBufPtr, inputBuf);
  const outputBufSizePtr = wasm.call('bbmalloc', 4);

  // Get a string version of our object. As a quick and dirty test,
  // we compare a snapshot of its string form to its previous form.
  const outputBufPtr = wasm.call(serializeMethod, inputBufPtr, outputBufSizePtr);

  // Read the size pointer
  const outputBufSize = uint8ArrayToNum(wasm.getMemorySlice(outputBufSizePtr, outputBufSizePtr + 4));

  // Copy into our own buffer
  const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));

  // Free memory
  wasm.call('bbfree', outputBufPtr);
  wasm.call('bbfree', outputBufSizePtr);
  wasm.call('bbfree', inputBufPtr);

  return outputBuf;
}

/**
 * Test utility. Checks a buffer serialize against a snapshot.
 * @param inputBuf - Buffer to write to.
 * @param serializeMethod - Method to use buffer with.
 * @param wasm - Optional circuit wasm. If not set, we fetch a singleton.
 */
export async function expectSerializeToMatchSnapshot(inputBuf: Buffer, serializeMethod: string, wasm?: CircuitsWasm) {
  const outputBuf = await callWasm(inputBuf, serializeMethod, wasm);
  const outputStr = simplifyHexValues(Buffer.from(outputBuf).toString('utf-8'));
  expect(outputStr).toMatchSnapshot();
}

/**
 * Test utility. Serializes an object, passes it to a wasm reserialize method,
 * gets it back, deserializes it, and checks it matches the original.
 * @param inputObj - Object to check.
 * @param serializeMethod - Wasm method to send and get back the object.
 * @param deserialize - Method to deserialize the object with.
 * @param wasm - Optional circuit wasm. If not set, we fetch a singleton.
 */
export async function expectReserializeToMatchObject<
  T extends {
    /**
     * Signature of the target serialization function.
     */
    toBuffer: () => Buffer;
  },
>(inputObj: T, serializeMethod: string, deserialize: (buf: Buffer) => T, wasm?: CircuitsWasm) {
  const outputBuf = await callWasm(inputObj.toBuffer(), serializeMethod, wasm);
  const deserializedObj = deserialize(outputBuf);
  expect(deserializedObj).toEqual(deserializedObj);
}
