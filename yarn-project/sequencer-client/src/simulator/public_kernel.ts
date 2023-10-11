import { PublicKernelInputs, PublicKernelPublicInputs, simulatePublicKernelCircuit } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';

import { PublicKernelCircuitSimulator } from './index.js';

/**
 * Implements the PublicKernelCircuitSimulator by calling the wasm implementations of the circuits.
 */
export class WasmPublicKernelCircuitSimulator implements PublicKernelCircuitSimulator {
  private log = createDebugLogger('aztec:public-kernel-simulator');

  /**
   * Simulates the public kernel circuit (with a previous private kernel circuit run) from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async publicKernelCircuitPrivateInput(input: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    if (!input.previousKernel.publicInputs.isPrivate) throw new Error(`Expected private kernel previous inputs`);
    const [duration, result] = await elapsed(() => simulatePublicKernelCircuit(input));
    this.log(`Simulated public kernel circuit with private input`, {
      eventName: 'circuit-simulation',
      circuitName: 'public-kernel-private-input',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    });
    return result;
  }

  /**
   * Simulates the public kernel circuit (with no previous public kernel circuit run) from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async publicKernelCircuitNonFirstIteration(input: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    if (input.previousKernel.publicInputs.isPrivate) throw new Error(`Expected public kernel previous inputs`);
    const [duration, result] = await elapsed(() => simulatePublicKernelCircuit(input));
    this.log(`Simulated public kernel circuit non-first iteration`, {
      eventName: 'circuit-simulation',
      circuitName: 'public-kernel-non-first-iteration',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    });
    return result;
  }
}
