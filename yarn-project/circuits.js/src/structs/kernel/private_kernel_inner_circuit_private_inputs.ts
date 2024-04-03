import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { PrivateCallData } from './private_call_data.js';
import { PrivateKernelData } from './private_kernel_data.js';

/**
 * Input to the private kernel circuit - Inner call.
 */
export class PrivateKernelInnerCircuitPrivateInputs {
  constructor(
    /**
     * The previous kernel data
     */
    public previousKernel: PrivateKernelData,
    /**
     * Private calldata corresponding to this iteration of the kernel.
     */
    public privateCall: PrivateCallData,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.privateCall);
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelInnerCircuitPrivateInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInnerCircuitPrivateInputs(
      reader.readObject(PrivateKernelData),
      reader.readObject(PrivateCallData),
    );
  }
}
