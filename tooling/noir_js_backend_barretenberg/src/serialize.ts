import { decompressSync as gunzip } from "fflate";
import { base64Decode } from "./base64_decode.js";

// Converts an bytecode to a Uint8Array
export function acirToUint8Array(base64EncodedBytecode): Uint8Array {
  const compressedByteCode = base64Decode(base64EncodedBytecode);
  return gunzip(compressedByteCode);
}
