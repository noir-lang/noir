"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Noir = void 0;
const witness_generation_js_1 = require("./witness_generation.cjs");
class Noir {
    circuit;
    backend;
    constructor(circuit, backend) {
        this.circuit = circuit;
        this.backend = backend;
    }
    // Initial inputs to your program
    async generateFinalProof(inputs) {
        const serializedWitness = await (0, witness_generation_js_1.generateWitness)(this.circuit, inputs);
        return this.backend.generateFinalProof(serializedWitness);
    }
    async verifyFinalProof(proof) {
        return this.backend.verifyFinalProof(proof);
    }
}
exports.Noir = Noir;
