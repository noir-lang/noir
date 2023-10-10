/* eslint-disable  @typescript-eslint/no-explicit-any */
import { Backend, CompiledCircuit, ProofData } from '@noir-lang/types';
import initAbi, * as abi from '@noir-lang/noirc_abi';
import initACVM, * as acvm from '@noir-lang/acvm_js';
import { base64Decode } from './base64_decode.js';
import { witnessMapToUint8Array } from './serialize.js';

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

  // Initial inputs to your program
  async generateFinalProof(inputs: any): Promise<ProofData> {
    if (!this.backend) throw new Error('Cannot generate proofs without a backend');

    await this.init();
    const serializedWitness = await this.generateWitness(this.backend.circuit, inputs);
    return this.backend.generateFinalProof(serializedWitness);
  }

  async verifyFinalProof(proofData: ProofData): Promise<boolean> {
    if (!this.backend) throw new Error('Cannot verify proofs without a backend');

    return this.backend.verifyFinalProof(proofData);
  }

  async generateWitness(circuit: CompiledCircuit, inputs: abi.InputMap) {
    // Throws on ABI encoding error
    const witnessMap = abi.abiEncode(circuit.abi, inputs);

    // Execute the circuit to generate the rest of the witnesses and serialize
    // them into a Uint8Array.
    try {
      const solvedWitness = await acvm.executeCircuit(base64Decode(circuit.bytecode), witnessMap, () => {
        throw Error('unexpected oracle during execution');
      });
      return witnessMapToUint8Array(solvedWitness);
    } catch (err) {
      throw new Error(`Circuit execution failed: ${err}`);
    }
  }
}

export { Noir, abi, acvm };
