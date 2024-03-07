import { serializeToBuffer } from '@aztec/foundation/serialize';

import { NullifierReadRequestResetHints } from '../read_request_reset_hints.js';
import { PublicKernelData } from './public_kernel_data.js';

/**
 * Inputs to the public kernel circuit.
 */
export class PublicKernelTailCircuitPrivateInputs {
  constructor(
    /**
     * Kernels are recursive and this is the data from the previous kernel.
     */
    public readonly previousKernel: PublicKernelData,
    /**
     * Contains hints for the nullifier read requests to locate corresponding pending or settled nullifiers.
     */
    public nullifierReadRequestResetHints: NullifierReadRequestResetHints,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.nullifierReadRequestResetHints);
  }
}
