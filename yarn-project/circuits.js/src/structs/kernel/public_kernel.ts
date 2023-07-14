import { Fr } from '@aztec/foundation/fields';
import { Tuple } from '@aztec/foundation/serialize';

import {
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  PUBLIC_DATA_TREE_HEIGHT,
} from '../../cbind/constants.gen.js';
import { assertMemberLength } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { PublicCallStackItem } from '../call_stack_item.js';
import { MembershipWitness } from '../membership_witness.js';
import { Proof } from '../proof.js';
import { PreviousKernelData } from './previous_kernel_data.js';

/**
 * Inputs to the public kernel circuit.
 */
export class PublicKernelInputs {
  constructor(
    /**
     * Kernels are recursive and this is the data from the previous kernel.
     */
    public readonly previousKernel: PreviousKernelData,
    /**
     * Public calldata assembled from the execution result and proof.
     */
    public readonly publicCall: PublicCallData,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.publicCall);
  }
}

/**
 // eslint-disable-next-line tsdoc/syntax
 * TODO: POSSIBLY OBSOLETE --\> DELETE OR DOCUMENT.
 */
export class WitnessedPublicCallData {
  constructor(
    /**
     * TODO.
     */
    public readonly publicCall: PublicCallData,
    /**
     * TODO.
     */
    public readonly updateRequestsHashPaths: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>[],
    /**
     * TODO.
     */
    public readonly readsHashPaths: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>[],
    /**
     * TODO.
     */
    public readonly publicDataTreeRoot: Fr,
  ) {
    assertMemberLength(this, 'updateRequestsHashPaths', MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX);
    assertMemberLength(this, 'readsHashPaths', MAX_PUBLIC_DATA_READS_PER_TX);
  }

  toBuffer() {
    return serializeToBuffer(
      this.publicCall,
      this.updateRequestsHashPaths,
      this.readsHashPaths,
      this.publicDataTreeRoot,
    );
  }
}

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
    public readonly publicCallStackPreimages: Tuple<PublicCallStackItem, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
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
  ) {
    assertMemberLength(this, 'publicCallStackPreimages', MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
  }

  toBuffer() {
    return serializeToBuffer(
      this.callStackItem,
      this.publicCallStackPreimages,
      this.proof,
      this.portalContractAddress,
      this.bytecodeHash,
    );
  }
}
