import {
  PublicKernelInputs,
  PublicKernelInputsNoPreviousKernel,
  PublicKernelPublicInputs,
  simulatePublicKernelCircuit,
  simulatePublicKernelCircuitNoPreviousKernel,
} from '@aztec/circuits.js';
import { PublicKernelCircuitSimulator } from './index.js';

export class WasmPublicKernelCircuitSimulator implements PublicKernelCircuitSimulator {
  publicKernelCircuitNoInput(inputs: PublicKernelInputsNoPreviousKernel): Promise<PublicKernelPublicInputs> {
    return simulatePublicKernelCircuitNoPreviousKernel(inputs);
  }
  publicKernelCircuitPrivateInput(inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    if (!inputs.previousKernel.publicInputs.isPrivateKernel) throw new Error(`Expected private kernel previous inputs`);
    return simulatePublicKernelCircuit(inputs);
  }
  publicKernelCircuitNonFirstIteration(inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    if (inputs.previousKernel.publicInputs.isPrivateKernel) throw new Error(`Expected public kernel previous inputs`);
    return simulatePublicKernelCircuit(inputs);
  }
}
