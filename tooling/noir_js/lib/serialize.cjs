"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.acirToUint8Array = exports.witnessMapToUint8Array = void 0;
const acvm_js_1 = require("@noir-lang/acvm_js");
const fflate_1 = require("fflate");
const base64_decode_js_1 = require("./base64_decode.cjs");
// After solving the witness, to pass it a backend, we need to serialize it to a Uint8Array
function witnessMapToUint8Array(solvedWitness) {
    // TODO: We just want to serialize, but this will zip up the witness
    // TODO so its not ideal
    const compressedWitness = (0, acvm_js_1.compressWitness)(solvedWitness);
    return (0, fflate_1.decompressSync)(compressedWitness);
}
exports.witnessMapToUint8Array = witnessMapToUint8Array;
// Converts an bytecode to a Uint8Array
function acirToUint8Array(base64EncodedBytecode) {
    const compressedByteCode = (0, base64_decode_js_1.base64Decode)(base64EncodedBytecode);
    return (0, fflate_1.decompressSync)(compressedByteCode);
}
exports.acirToUint8Array = acirToUint8Array;
