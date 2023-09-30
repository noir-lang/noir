/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend, CompiledCircuit } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';
import initNoirAbi from '@noir-lang/noirc_abi';

export class Noir {
  constructor(
    private circuit: CompiledCircuit,
    private backend: Backend,
  ) {
    initNoirAbi();
  }

  // Initial inputs to your program
  async generateFinalProof(inputs: any): Promise<Uint8Array> {
    const serializedWitness = await generateWitness(this.circuit, inputs);
    return this.backend.generateFinalProof(serializedWitness);
  }

  async verifyFinalProof(proof: Uint8Array): Promise<boolean> {
    return this.backend.verifyFinalProof(proof);
  }
}
