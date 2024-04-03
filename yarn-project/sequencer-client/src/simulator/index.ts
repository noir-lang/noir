import {
  type KernelCircuitPublicInputs,
  type PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  type PublicKernelTailCircuitPrivateInputs,
} from '@aztec/circuits.js';

/**
 * Circuit simulator for the public kernel circuits.
 */
export interface PublicKernelCircuitSimulator {
  /**
   * Simulates the public kernel setup circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitSetup(inputs: PublicKernelCircuitPrivateInputs): Promise<PublicKernelCircuitPublicInputs>;
  /**
   * Simulates the public kernel app logic circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitAppLogic(inputs: PublicKernelCircuitPrivateInputs): Promise<PublicKernelCircuitPublicInputs>;
  /**
   * Simulates the public kernel teardown circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitTeardown(inputs: PublicKernelCircuitPrivateInputs): Promise<PublicKernelCircuitPublicInputs>;
  /**
   * Simulates the public kernel tail circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitTail(inputs: PublicKernelTailCircuitPrivateInputs): Promise<KernelCircuitPublicInputs>;
}
