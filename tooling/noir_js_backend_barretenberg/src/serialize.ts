import { decompressSync as gunzip } from 'fflate';
import { base64Decode } from './base64_decode.js';

// Converts bytecode from a base64 string to a Uint8Array
export function acirToUint8Array(base64EncodedBytecode: string): Uint8Array {
  const compressedByteCode = base64Decode(base64EncodedBytecode);
  return gunzip(compressedByteCode);
}
