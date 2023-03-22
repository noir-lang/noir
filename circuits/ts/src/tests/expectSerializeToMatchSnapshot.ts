import { CircuitsWasm } from "../wasm/circuits_wasm.js";
import { uint8ArrayToNum } from "../wasm/serialize.js";

/**
 * Simplify e.g. 0x0003 into 0x3.
 * @param input - The input string, with hex somewhere inside.
 * @returns The output string with fixed hex.
 */
function simplifyHexValues(input: string) {
  const regex = /0x[\dA-Fa-f]+/g;
  const matches = input.match(regex) || [];
  const simplifiedMatches = matches.map(
    (match) => "0x" + BigInt(match).toString(16)
  );
  const result = input.replace(regex, () => simplifiedMatches.shift() || "");
  return result;
}

/**
 * Test utility. Checks a buffer serialize against a snapshot.
 * @param inputBuf - Buffer to write.
 * @param serializeMethod - Method to use buffer with.
 */
export async function expectSerializeToMatchSnapshot(
  inputBuf: Buffer,
  serializeMethod: string,
  wasm?: CircuitsWasm
) {
  wasm = wasm || (await CircuitsWasm.new());
  const inputBufPtr = wasm.call("bbmalloc", inputBuf.length);
  wasm.writeMemory(inputBufPtr, inputBuf);
  const outputBufSizePtr = wasm.call("bbmalloc", 4);
  // Get a string version of our object. As a quick and dirty test,
  // we compare a snapshot of its string form to its previous form.
  const outputBufPtr = wasm.call(
    serializeMethod,
    inputBufPtr,
    outputBufSizePtr
  );
  // Read the size pointer
  const outputBufSize = uint8ArrayToNum(
    wasm.getMemorySlice(outputBufSizePtr, outputBufSizePtr + 4)
  );
  const outputBuf = wasm.getMemorySlice(
    outputBufPtr,
    outputBufPtr + outputBufSize
  );
  const outputStr = simplifyHexValues(Buffer.from(outputBuf).toString("utf-8"));
  expect(outputStr).toMatchSnapshot();
  // Free memory
  wasm.call("bbfree", outputBufPtr);
  wasm.call("bbfree", outputBufSizePtr);
  wasm.call("bbfree", inputBufPtr);
}
