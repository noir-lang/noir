import { CompiledCircuit } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';
import initAbi, { abiDecode, InputMap, InputValue } from '@noir-lang/noirc_abi';
import initACVM, { compressWitnessStack, ForeignCallHandler } from '@noir-lang/acvm_js';

export class Noir {
  constructor(private circuit: CompiledCircuit) {}

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
   * @description
   * Allows to execute a circuit to get its witness and return value.
   *
   * @example
   * ```typescript
   * async execute(inputs)
   * ```
   */
  async execute(
    inputs: InputMap,
    foreignCallHandler?: ForeignCallHandler,
  ): Promise<{ witness: Uint8Array; returnValue: InputValue }> {
    await this.init();
    const witness_stack = await generateWitness(this.circuit, inputs, foreignCallHandler);
    const main_witness = witness_stack[0].witness;
    const { return_value: returnValue } = abiDecode(this.circuit.abi, main_witness);
    return { witness: compressWitnessStack(witness_stack), returnValue };
  }
}
