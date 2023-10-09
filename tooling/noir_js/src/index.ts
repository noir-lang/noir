/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend, ProofData } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';
import initAbi from '@noir-lang/noirc_abi';
import initACVM from '@noir-lang/acvm_js';

export class Noir {
  constructor(private backend: Backend) {}

  async init(): Promise<void> {
    // If these are available, then we are in the
    // web environment. For the node environment, this
    // is a no-op.
    if (typeof initAbi === 'function') {
      await Promise.all([initAbi(), initACVM()]);
    }
    await this.backend.instantiate();
  }

  async destroy(): Promise<void> {
    await this.backend.destroy();
  }

  // Initial inputs to your program
  async generateFinalProof(inputs: any): Promise<ProofData> {
    await this.init();
    const serializedWitness = await generateWitness(this.backend.circuit, inputs);
    return this.backend.generateFinalProof(serializedWitness);
  }

  async verifyFinalProof(proofData: ProofData): Promise<boolean> {
    return this.backend.verifyFinalProof(proofData);
  }
}
