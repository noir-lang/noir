import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_NEW_NOTE_HASHES_PER_CALL } from '../../constants.gen.js';
import { PrivateCallData } from './private_call_data.js';
import { PrivateKernelData } from './private_kernel_data.js';

export class PrivateKernelInnerHints {
  constructor(public noteHashNullifierCounters: Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_CALL>) {}

  toBuffer() {
    return serializeToBuffer(this.noteHashNullifierCounters);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInnerHints(reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_CALL));
  }
}

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
    public hints: PrivateKernelInnerHints,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.privateCall, this.hints);
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
      reader.readObject(PrivateKernelInnerHints),
    );
  }
}
