import { type Fr } from '@aztec/foundation/fields';
import { type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { type MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL } from '../../constants.gen.js';
import { type CallRequest } from '../call_request.js';
import { type Proof } from '../proof.js';
import { type PublicCallStackItem } from '../public_call_stack_item.js';

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
     * Address of the corresponding portal contract.
     */
    public readonly portalContractAddress: Fr,
    /**
     * Hash of the L2 contract bytecode.
     */
    public readonly bytecodeHash: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.callStackItem,
      this.publicCallStack,
      this.proof,
      this.portalContractAddress,
      this.bytecodeHash,
    );
  }
}
