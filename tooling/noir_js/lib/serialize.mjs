import { compressWitness } from '@noir-lang/acvm_js';
import { decompressSync as gunzip } from 'fflate';
import { base64Decode } from "./base64_decode.mjs";
// After solving the witness, to pass it a backend, we need to serialize it to a Uint8Array
export function witnessMapToUint8Array(solvedWitness) {
    // TODO: We just want to serialize, but this will zip up the witness
    // TODO so its not ideal
    const compressedWitness = compressWitness(solvedWitness);
    return gunzip(compressedWitness);
}
// Converts an bytecode to a Uint8Array
export function acirToUint8Array(base64EncodedBytecode) {
    const compressedByteCode = base64Decode(base64EncodedBytecode);
    return gunzip(compressedByteCode);
}
