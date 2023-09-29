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
exports.Noir = exports.witnessMapToUint8Array = exports.acirToUint8Array = exports.generateWitness = exports.abi = exports.acvm = void 0;
const acvm = __importStar(require("@noir-lang/acvm_js"));
exports.acvm = acvm;
const abi = __importStar(require("@noir-lang/noirc_abi"));
exports.abi = abi;
var witness_generation_js_1 = require("./witness_generation.cjs");
Object.defineProperty(exports, "generateWitness", { enumerable: true, get: function () { return witness_generation_js_1.generateWitness; } });
var serialize_js_1 = require("./serialize.cjs");
Object.defineProperty(exports, "acirToUint8Array", { enumerable: true, get: function () { return serialize_js_1.acirToUint8Array; } });
Object.defineProperty(exports, "witnessMapToUint8Array", { enumerable: true, get: function () { return serialize_js_1.witnessMapToUint8Array; } });
var program_js_1 = require("./program.cjs");
Object.defineProperty(exports, "Noir", { enumerable: true, get: function () { return program_js_1.Noir; } });
