import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

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
     * Proof of the call stack item execution.
     */
    public readonly proof: Proof,
    /**
     * Hash of the L2 contract bytecode.
     */
    public readonly bytecodeHash: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.callStackItem, this.proof, this.bytecodeHash);
  }

  static fromBuffer(buffer: BufferReader | Buffer) {
    const reader = BufferReader.asReader(buffer);
    return new PublicCallData(reader.readObject(PublicCallStackItem), reader.readObject(Proof), reader.readObject(Fr));
  }
}
