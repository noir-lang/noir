import {
  PublicKernelInputsNoKernelInput,
  PublicKernelPublicInputs,
  PublicKernelInputsPrivateKernelInput,
  PublicKernelInputsNonFirstIteration,
} from '@aztec/circuits.js';
import { PublicKernelCircuitSimulator } from './index.js';

export class MockPublicKernelCircuitSimulator implements PublicKernelCircuitSimulator {
  publicKernelCircuitNoInput(_inputs: PublicKernelInputsNoKernelInput): Promise<PublicKernelPublicInputs> {
    return Promise.resolve(PublicKernelPublicInputs.empty());
  }
  publicKernelCircuitPrivateInput(_inputs: PublicKernelInputsPrivateKernelInput): Promise<PublicKernelPublicInputs> {
    return Promise.resolve(PublicKernelPublicInputs.empty());
  }
  publicKernelCircuitNonFirstIteration(
    _inputs: PublicKernelInputsNonFirstIteration,
  ): Promise<PublicKernelPublicInputs> {
    return Promise.resolve(PublicKernelPublicInputs.empty());
  }
}
