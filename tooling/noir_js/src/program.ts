/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend } from './backend/backend_interface.js';
import { generateWitness } from './witness_generation.js';

export class Noir {
  constructor(
    private circuit: { bytecode: string; abi: any },
    private backend: Backend,
  ) {}

  // Initial inputs to your program
  async generateFinalProof(inputs: any): Promise<Uint8Array> {
    const serializedWitness = await generateWitness(this.circuit, inputs);
    return this.backend.generateFinalProof(serializedWitness);
  }

  async verifyFinalProof(proof: Uint8Array): Promise<boolean> {
    return this.backend.verifyFinalProof(proof);
  }
}
