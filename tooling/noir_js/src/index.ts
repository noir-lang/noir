/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend, CompiledCircuit, ProofData } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';
import initAbi, * as abi from '@noir-lang/noirc_abi';
import initACVM, * as acvm from '@noir-lang/acvm_js';
import { witnessMapToUint8Array } from './serialize.js';

const { abiDecode } = abi;

class Noir {
  constructor(private backend?: Backend) {}

  async init(): Promise<void> {
    // If these are available, then we are in the
    // web environment. For the node environment, this
    // is a no-op.
    if (typeof initAbi === 'function') {
      await Promise.all([initAbi(), initACVM()]);
    }
  }

  async destroy(): Promise<void> {
    if (!this.backend) throw new Error('No backend to destroy');

    await this.backend.destroy();
  }

  private getBackend(): Backend {
    if (this.backend === undefined) throw new Error('Operation requires a backend but none was provided');
    return this.backend;
  }

  // Initial inputs to your program
  async execute(
    inputs: abi.InputMap,
    circuit?: CompiledCircuit,
  ): Promise<{ witness: Uint8Array; returnValue: abi.InputValue }> {
    if (!circuit && !this.backend) throw new Error('Operation requires a circuit or a backend, but none was provided');

    await this.init();
    const witness = await generateWitness(circuit!, inputs);
    const { return_value: returnValue } = abiDecode(circuit!.abi, witness);
    return { witness: witnessMapToUint8Array(witness), returnValue };
  }

  // Initial inputs to your program
  async generateFinalProof(inputs: abi.InputMap): Promise<ProofData> {
    if (!this.backend) throw new Error('Operation requires a backend but none was provided');

    const { witness } = await this.execute(inputs, this.backend.circuit);
    return this.getBackend().generateFinalProof(witness);
  }

  async verifyFinalProof(proofData: ProofData): Promise<boolean> {
    return this.getBackend().verifyFinalProof(proofData);
  }
}

export { Noir, acvm, abi };
