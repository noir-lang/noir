/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend, CompiledCircuit, ProofData } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';
import initAbi, { abiDecode, InputMap, InputValue } from '@noir-lang/noirc_abi';
import initACVM, { compressWitness } from '@noir-lang/acvm_js';

export class Noir {
  constructor(
    private circuit: CompiledCircuit,
    private backend?: Backend,
  ) {}

  async init(): Promise<void> {
    // If these are available, then we are in the
    // web environment. For the node environment, this
    // is a no-op.
    if (typeof initAbi === 'function') {
      await Promise.all([initAbi(), initACVM()]);
    }
  }

  async destroy(): Promise<void> {
    await this.backend?.destroy();
  }

  private getBackend(): Backend {
    if (this.backend === undefined) throw new Error('Operation requires a backend but none was provided');
    return this.backend;
  }

  // Initial inputs to your program
  async execute(inputs: InputMap): Promise<{ witness: Uint8Array; returnValue: InputValue }> {
    await this.init();
    const witness = await generateWitness(this.circuit, inputs);
    const { return_value: returnValue } = abiDecode(this.circuit.abi, witness);
    return { witness: compressWitness(witness), returnValue };
  }

  // Initial inputs to your program
  async generateFinalProof(inputs: InputMap): Promise<ProofData> {
    const { witness } = await this.execute(inputs);
    return this.getBackend().generateFinalProof(witness);
  }

  async verifyFinalProof(proofData: ProofData): Promise<boolean> {
    return this.getBackend().verifyFinalProof(proofData);
  }
}
