import { serializeToBuffer } from '@aztec/foundation/serialize';

import { type NullifierNonExistentReadRequestHints } from '../non_existent_read_request_hints.js';
import { type NullifierReadRequestHints } from '../read_request_hints.js';
import { type PublicKernelData } from './public_kernel_data.js';

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
    public readonly nullifierReadRequestHints: NullifierReadRequestHints,
    /**
     * Contains hints for the nullifier non existent read requests.
     */
    public readonly nullifierNonExistentReadRequestHints: NullifierNonExistentReadRequestHints,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.previousKernel,
      this.nullifierReadRequestHints,
      this.nullifierNonExistentReadRequestHints,
    );
  }
}
