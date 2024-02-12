import { serializeToBuffer } from '@aztec/foundation/serialize';

import { PublicCallData } from './public_call_data.js';
import { PublicKernelData } from './public_kernel_data.js';

/**
 * Inputs to the public kernel circuit.
 */
export class PublicKernelCircuitPrivateInputs {
  constructor(
    /**
     * Kernels are recursive and this is the data from the previous kernel.
     */
    public readonly previousKernel: PublicKernelData,
    /**
     * Public calldata assembled from the execution result and proof.
     */
    public readonly publicCall: PublicCallData,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.publicCall);
  }
}
