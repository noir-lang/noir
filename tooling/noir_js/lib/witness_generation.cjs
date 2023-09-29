"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.generateWitness = void 0;
const noirc_abi_1 = __importStar(require("@noir-lang/noirc_abi"));
const base64_decode_js_1 = require("./base64_decode.cjs");
const acvm_js_1 = __importStar(require("@noir-lang/acvm_js"));
const serialize_js_1 = require("./serialize.cjs");
// Generates the witnesses needed to feed into the chosen proving system
async function generateWitness(compiledProgram, inputs) {
    if (typeof noirc_abi_1.default === 'function' && typeof acvm_js_1.default === 'function') {
        await (0, noirc_abi_1.default)();
        await (0, acvm_js_1.default)();
    }
    // Throws on ABI encoding error
    const witnessMap = (0, noirc_abi_1.abiEncode)(compiledProgram.abi, inputs, null);
    // Execute the circuit to generate the rest of the witnesses and serialize
    // them into a Uint8Array.
    try {
        const solvedWitness = await (0, acvm_js_1.executeCircuit)((0, base64_decode_js_1.base64Decode)(compiledProgram.bytecode), witnessMap, () => {
            throw Error('unexpected oracle during execution');
        });
        return (0, serialize_js_1.witnessMapToUint8Array)(solvedWitness);
    }
    catch (err) {
        throw new Error(`Circuit execution failed: ${err}`);
    }
}
exports.generateWitness = generateWitness;
