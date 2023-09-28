/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend, CompiledCircuit } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';

export class Program {
  constructor(
    private circuit: CompiledCircuit,
    private backend: Backend,
  ) {}

  // Initial inputs to your program
  async generateProof(inputs: any): Promise<Uint8Array> {
    const serializedWitness = await generateWitness(this.circuit, inputs);
    return this.backend.generateProof(serializedWitness);
  }

  async verifyProof(proof: Uint8Array): Promise<boolean> {
    return this.backend.verifyProof(proof);
  }
}
