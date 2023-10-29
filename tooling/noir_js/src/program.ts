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

  /** @ignore */
  async init(): Promise<void> {
    // If these are available, then we are in the
    // web environment. For the node environment, this
    // is a no-op.
    if (typeof initAbi === 'function') {
      await Promise.all([initAbi(), initACVM()]);
    }
  }

  /**
   *
   * @description
   * Destroys the underlying backend instance.
   *
   * @example
   * ```typescript
   * await noir.destroy();
   * ```
   *
   */
  async destroy(): Promise<void> {
    await this.backend?.destroy();
  }

  private getBackend(): Backend {
    if (this.backend === undefined) throw new Error('Operation requires a backend but none was provided');
    return this.backend;
  }

  /**
   * @description
   * Allows to execute a circuit to get its witness and return value.
   *
   * @example
   * ```typescript
   * async execute(inputs)
   * ```
   */
  async execute(inputs: InputMap): Promise<{ witness: Uint8Array; returnValue: InputValue }> {
    await this.init();
    const witness = await generateWitness(this.circuit, inputs);
    const { return_value: returnValue } = abiDecode(this.circuit.abi, witness);
    return { witness: compressWitness(witness), returnValue };
  }

  /**
   *
   * @description
   * Generates a witness and a proof given an object as input.
   *
   * @example
   * ```typescript
   * async generateFinalproof(input)
   * ```
   *
   */
  async generateFinalProof(inputs: InputMap): Promise<ProofData> {
    const { witness } = await this.execute(inputs);
    return this.getBackend().generateFinalProof(witness);
  }

  /**
   *
   * @description
   * Instantiates the verification key and verifies a proof.
   *
   *
   * @example
   * ```typescript
   * async verifyFinalProof(proof)
   * ```
   *
   */
  async verifyFinalProof(proofData: ProofData): Promise<boolean> {
    return this.getBackend().verifyFinalProof(proofData);
  }
}
