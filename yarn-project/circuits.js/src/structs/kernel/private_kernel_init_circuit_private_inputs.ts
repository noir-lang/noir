import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { TxRequest } from '../tx_request.js';
import { PrivateCallData } from './private_call_data.js';
import { PrivateKernelInnerHints } from './private_kernel_inner_circuit_private_inputs.js';

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
    public hints: PrivateKernelInnerHints,
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
      reader.readObject(PrivateKernelInnerHints),
    );
  }
}
