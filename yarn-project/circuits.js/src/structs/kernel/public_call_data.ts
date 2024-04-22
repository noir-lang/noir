import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL } from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { Proof } from '../proof.js';
import { PublicCallStackItem } from '../public_call_stack_item.js';

/**
 * Public calldata assembled from the kernel execution result and proof.
 */
export class PublicCallData {
  constructor(
    /**
     * Call stack item being processed by the current iteration of the kernel.
     */
    public readonly callStackItem: PublicCallStackItem,
    /**
     * Children call stack items.
     */
    public readonly publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * Proof of the call stack item execution.
     */
    public readonly proof: Proof,
    /**
     * Hash of the L2 contract bytecode.
     */
    public readonly bytecodeHash: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.callStackItem, this.publicCallStack, this.proof, this.bytecodeHash);
  }

  static fromBuffer(buffer: BufferReader | Buffer) {
    const reader = BufferReader.asReader(buffer);
    return new PublicCallData(
      reader.readObject(PublicCallStackItem),
      reader.readArray<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>(
        MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
        CallRequest,
      ),
      reader.readObject(Proof),
      reader.readObject(Fr),
    );
  }
}
