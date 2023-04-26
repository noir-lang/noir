import { Fr } from '@aztec/foundation';
import { assertLength } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { PublicCallStackItem } from '../call_stack_item.js';
import {
  PUBLIC_CALL_STACK_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
  STATE_READS_LENGTH,
  STATE_TRANSITIONS_LENGTH,
} from '../constants.js';
import { MembershipWitness } from '../membership_witness.js';
import { UInt8Vector } from '../shared.js';
import { SignedTxRequest } from '../tx_request.js';
import { PreviousKernelData } from './previous_kernel_data.js';

export class PublicKernelInputs {
  constructor(public readonly previousKernel: PreviousKernelData, public readonly publicCallData: PublicCallData) {}

  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.publicCallData);
  }
}
export class PublicKernelInputsNoPreviousKernel {
  public kind = 'NoKernelInput' as const;

  constructor(public readonly signedTxRequest: SignedTxRequest, public readonly publicCallData: PublicCallData) {}

  toBuffer() {
    return serializeToBuffer(this.signedTxRequest, this.publicCallData);
  }
}

export class WitnessedPublicCallData {
  constructor(
    public readonly publicCall: PublicCallData,
    public readonly transitionsHashPaths: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>[],
    public readonly readsHashPaths: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>[],
    public readonly publicDataTreeRoot: Fr,
  ) {
    assertLength(this, 'transitionsHashPaths', STATE_TRANSITIONS_LENGTH);
    assertLength(this, 'readsHashPaths', STATE_READS_LENGTH);
  }

  toBuffer() {
    return serializeToBuffer(this.publicCall, this.transitionsHashPaths, this.readsHashPaths, this.publicDataTreeRoot);
  }
}

export class PublicCallData {
  constructor(
    public readonly callStackItem: PublicCallStackItem,
    public readonly publicCallStackPreimages: PublicCallStackItem[],
    public readonly proof: UInt8Vector,
    public readonly portalContractAddress: Fr,
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
