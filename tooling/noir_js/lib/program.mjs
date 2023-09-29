import { generateWitness } from "./witness_generation.mjs";
export class Noir {
    circuit;
    backend;
    constructor(circuit, backend) {
        this.circuit = circuit;
        this.backend = backend;
    }
    // Initial inputs to your program
    async generateFinalProof(inputs) {
        const serializedWitness = await generateWitness(this.circuit, inputs);
        return this.backend.generateFinalProof(serializedWitness);
    }
    async verifyFinalProof(proof) {
        return this.backend.verifyFinalProof(proof);
    }
}
