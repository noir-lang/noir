import { generateWitness } from "./witness_generation.mjs";
export class Program {
    circuit;
    backend;
    constructor(circuit, backend) {
        this.circuit = circuit;
        this.backend = backend;
    }
    // Initial inputs to your program
    async generateProof(inputs, optimizeForVerifyInCircuit = false) {
        const serializedWitness = await generateWitness(this.circuit, inputs);
        return this.backend.generateProof(serializedWitness, optimizeForVerifyInCircuit);
    }
    async verifyProof(proof, optimizeForVerifyInCircuit = false) {
        return this.backend.verifyProof(proof, optimizeForVerifyInCircuit);
    }
}
