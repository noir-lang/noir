import { PublicKernelInputs, PublicKernelPublicInputs, simulatePublicKernelCircuit } from '@aztec/circuits.js';

import { PublicKernelCircuitSimulator } from './index.js';

/**
 * Implements the PublicKernelCircuitSimulator by calling the wasm implementations of the circuits.
 */
export class WasmPublicKernelCircuitSimulator implements PublicKernelCircuitSimulator {
  /**
   * Simulates the public kernel circuit (with a previous private kernel circuit run) from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public publicKernelCircuitPrivateInput(input: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    if (!input.previousKernel.publicInputs.isPrivate) throw new Error(`Expected private kernel previous inputs`);
    return simulatePublicKernelCircuit(input);
  }

  /**
   * Simulates the public kernel circuit (with no previous public kernel circuit run) from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitNonFirstIteration(input: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    if (input.previousKernel.publicInputs.isPrivate) throw new Error(`Expected public kernel previous inputs`);
    return simulatePublicKernelCircuit(input);
  }
}
