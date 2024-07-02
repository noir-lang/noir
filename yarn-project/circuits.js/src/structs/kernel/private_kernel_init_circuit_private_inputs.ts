import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_NEW_NOTE_HASHES_PER_CALL } from '../../constants.gen.js';
import { TxRequest } from '../tx_request.js';
import { PrivateCallData } from './private_call_data.js';

export class PrivateKernelInitHints {
  constructor(public noteHashNullifierCounters: Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_CALL>) {}

  toBuffer() {
    return serializeToBuffer(this.noteHashNullifierCounters);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInitHints(reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_CALL));
  }
}

/**
 * Input to the private kernel circuit - initial call.
 */
export class PrivateKernelInitCircuitPrivateInputs {
  constructor(
    /**
     * The transaction request which led to the creation of these inputs.
     */
    public txRequest: TxRequest,
    /**
     * Private calldata corresponding to this iteration of the kernel.
     */
    public privateCall: PrivateCallData,
    public hints: PrivateKernelInitHints,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.txRequest, this.privateCall, this.hints);
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelInitCircuitPrivateInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInitCircuitPrivateInputs(
      reader.readObject(TxRequest),
      reader.readObject(PrivateCallData),
      reader.readObject(PrivateKernelInitHints),
    );
  }
}
