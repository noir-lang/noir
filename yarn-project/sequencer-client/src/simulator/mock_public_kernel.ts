import { PublicKernelInputsNoPreviousKernel, PublicKernelPublicInputs, PublicKernelInputs } from '@aztec/circuits.js';
import { PublicKernelCircuitSimulator } from './index.js';

export class MockPublicKernelCircuitSimulator implements PublicKernelCircuitSimulator {
  publicKernelCircuitNoInput(_inputs: PublicKernelInputsNoPreviousKernel): Promise<PublicKernelPublicInputs> {
    return Promise.resolve(PublicKernelPublicInputs.empty());
  }
  publicKernelCircuitPrivateInput(_inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    return Promise.resolve(PublicKernelPublicInputs.empty());
  }
  publicKernelCircuitNonFirstIteration(_inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs> {
    return Promise.resolve(PublicKernelPublicInputs.empty());
  }
}
