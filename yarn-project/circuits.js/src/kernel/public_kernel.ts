import {
  CircuitsWasm,
  KernelCircuitPublicInputs,
  PublicKernelInputs,
  PublicKernelInputsNoPreviousKernel,
} from '../index.js';
import { callAsyncWasm } from '../utils/call_wasm.js';

export async function simulatePublicKernelCircuit(input: PublicKernelInputs): Promise<KernelCircuitPublicInputs> {
  return callAsyncWasm(await CircuitsWasm.get(), 'public_kernel__sim', input, KernelCircuitPublicInputs);
}

export async function simulatePublicKernelCircuitNoPreviousKernel(
  input: PublicKernelInputsNoPreviousKernel,
): Promise<KernelCircuitPublicInputs> {
  return callAsyncWasm(
    await CircuitsWasm.get(),
    'public_kernel_no_previous_kernel__sim',
    input,
    KernelCircuitPublicInputs,
  );
}
