import { publicKernelSim } from '../cbind/circuits.gen.js';
import { CircuitError, CircuitsWasm, KernelCircuitPublicInputs, PublicKernelInputs } from '../index.js';

/**
 * Computes the public inputs of the kernel circuit.
 * @param input - The kernel circuit inputs.
 * @returns The public inputs.
 */
export async function simulatePublicKernelCircuit(input: PublicKernelInputs): Promise<KernelCircuitPublicInputs> {
  const result = publicKernelSim(await CircuitsWasm.get(), input);
  if (result instanceof CircuitError) {
    result.message += '\nRefer to https://docs.aztec.network/aztec/protocol/errors for more information.';
    throw new CircuitError(result.code, result.message);
  }
  return result;
}
