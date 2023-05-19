import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { simplifyHexValues } from './expectSerialize.js';

/**
 * Test utility. Sends a serialized buffer to wasm and gets the result.
 * @param inputBuf - Buffer to write.
 * @param serializeMethod - Method to use buffer with.
 * @param wasm - Optional circuit wasm.
 */
async function callWasm(inputBuf: Buffer, serializeMethod: string): Promise<string> {
  const wasm = await CircuitsWasm.get();
  const inputBufPtr = wasm.call('bbmalloc', inputBuf.length);
  wasm.writeMemory(inputBufPtr, inputBuf);

  // Get a msgpack string version of our object. As a quick and dirty test,
  // we compare a snapshot of its string form to its previous form.
  const outputBufPtr = await wasm.asyncCall(serializeMethod, inputBufPtr);

  // Read the size pointer
  const outputStr = wasm.getMemoryAsString(outputBufPtr);

  // Free memory
  wasm.call('bbfree', outputBufPtr);

  return outputStr;
}

/**
 * Test utility. Checks a buffer serialize against a snapshot.
 * @param inputBuf - Buffer to write.
 * @param serializeMethod - Method to use buffer with.
 */
export async function expectMsgpackToMatchSnapshot(inputBuf: Buffer, serializeMethod: string) {
  const outputStr = simplifyHexValues(await callWasm(inputBuf, serializeMethod));
  expect(JSON.parse(outputStr)).toMatchSnapshot();
}
