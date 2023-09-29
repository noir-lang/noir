"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Program = void 0;
const witness_generation_js_1 = require("./witness_generation.cjs");
class Program {
    circuit;
    backend;
    constructor(circuit, backend) {
        this.circuit = circuit;
        this.backend = backend;
    }
    // Initial inputs to your program
    async generateProof(inputs, optimizeForVerifyInCircuit = false) {
        const serializedWitness = await (0, witness_generation_js_1.generateWitness)(this.circuit, inputs);
        return this.backend.generateProof(serializedWitness, optimizeForVerifyInCircuit);
    }
    async verifyProof(proof, optimizeForVerifyInCircuit = false) {
        return this.backend.verifyProof(proof, optimizeForVerifyInCircuit);
    }
}
exports.Program = Program;
