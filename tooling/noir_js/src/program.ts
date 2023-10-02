/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend, CompiledCircuit } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';
import initAbi from '@noir-lang/noirc_abi';
import initACVM from '@noir-lang/acvm_js';

export class Noir {
  constructor(
    private circuit: CompiledCircuit,
    private backend: Backend,
  ) {}

  async init(): Promise<void> {
    // If these are available, then we are in the
    // web environment. For the node environment, this
    // is a no-op.
    if (typeof initAbi === 'function') {
      await Promise.all([initAbi(), initACVM()]);
    }
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
