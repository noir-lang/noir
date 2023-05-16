import { Fr } from '@aztec/foundation/fields';
import { assertLength } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { PublicCallStackItem } from '../call_stack_item.js';
import {
  PUBLIC_CALL_STACK_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  KERNEL_PUBLIC_DATA_READS_LENGTH,
  KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
} from '../constants.js';
import { MembershipWitness } from '../membership_witness.js';
import { UInt8Vector } from '../shared.js';
import { SignedTxRequest } from '../tx_request.js';
import { PreviousKernelData } from './previous_kernel_data.js';
import { CombinedHistoricTreeRoots } from './combined_constant_data.js';

/**
 * Inputs to the public kernel circuit.
 */
export class PublicKernelInputs {
  constructor(
    /**
     * Kernels are recursive and this is the data from the previous kernel.
     * When this is the first kernel in the chain of kernels use `PublicKernelInputsNoPreviousKernel` instead.
     */
    public readonly previousKernel: PreviousKernelData,
    /**
     * Public calldata assembled from the execution result and proof.
     */
    public readonly publicCallData: PublicCallData,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.publicCallData);
  }
}
/**
 * Inputs to the public kernel circuit when there is no previous kernel.
 */
export class PublicKernelInputsNoPreviousKernel {
  /**
   * Kernel kind.
   */
  public kind = 'NoKernelInput' as const;

  constructor(
    /**
     * The tx request signed by the user when initiating the transaction.
     */
    public readonly signedTxRequest: SignedTxRequest,
    /**
     * Public calldata assembled from the kernel execution result and proof.
     */
    public readonly publicCallData: PublicCallData,
    /**
     * Constant data historic tree roots.
     */
    public readonly historicTreeRoots: CombinedHistoricTreeRoots,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.signedTxRequest, this.publicCallData, this.historicTreeRoots);
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
    assertLength(this, 'updateRequestsHashPaths', KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH);
    assertLength(this, 'readsHashPaths', KERNEL_PUBLIC_DATA_READS_LENGTH);
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
    public readonly publicCallStackPreimages: PublicCallStackItem[],
    /**
     * Proof of the call stack item execution.
     */
    public readonly proof: UInt8Vector,
    /**
     * Address of the corresponding portal contract.
     */
    public readonly portalContractAddress: Fr,
    /**
     * Hash of the L2 contract bytecode.
     */
    public readonly bytecodeHash: Fr,
  ) {
    assertLength(this, 'publicCallStackPreimages', PUBLIC_CALL_STACK_LENGTH);
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
