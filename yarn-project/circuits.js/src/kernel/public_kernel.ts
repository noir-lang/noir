import { publicKernelSim } from '../cbind/circuits.gen.js';
import {
  CircuitError,
  CircuitsWasm,
  KernelCircuitPublicInputs,
  PublicKernelInputs,
  PublicKernelInputsNoPreviousKernel,
} from '../index.js';
import { callWasm } from '../utils/call_wasm.js';

/**
 * Computes the public inputs of the kernel circuit.
 * @param input - The kernel circuit inputs.
 * @returns The public inputs.
 */
export async function simulatePublicKernelCircuit(input: PublicKernelInputs): Promise<KernelCircuitPublicInputs> {
  const result = publicKernelSim(await CircuitsWasm.get(), input);
  if (result instanceof CircuitError) {
    throw new CircuitError(result.code, result.message);
  }
  return result;
}

/**
 * Computes the public inputs of the kernel circuit when there is no previous kernel. Used for direct calls to public functions in a transaction.
 * @param input - The kernel circuit inputs.
 * @returns The public inputs.
 */
export async function simulatePublicKernelCircuitNoPreviousKernel(
  input: PublicKernelInputsNoPreviousKernel,
): Promise<KernelCircuitPublicInputs> {
  return callWasm(await CircuitsWasm.get(), 'public_kernel_no_previous_kernel__sim', input, KernelCircuitPublicInputs);
}
