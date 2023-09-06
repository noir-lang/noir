import { Tuple } from '@aztec/foundation/serialize';

import {
  CONTRACT_TREE_HEIGHT,
  FUNCTION_TREE_HEIGHT,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_TX,
} from '../../cbind/constants.gen.js';
import { FieldsOf, assertMemberLength } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { PrivateCallStackItem } from '../call_stack_item.js';
import { Fr } from '../index.js';
import { MembershipWitness } from '../membership_witness.js';
import { Proof } from '../proof.js';
import { ReadRequestMembershipWitness } from '../read_request_membership_witness.js';
import { TxRequest } from '../tx_request.js';
import { VerificationKey } from '../verification_key.js';
import { PreviousKernelData } from './previous_kernel_data.js';

/**
 * Private call data.
 * @see circuits/cpp/src/aztec3/circuits/abis/call_stack_item.hpp
 */
export class PrivateCallData {
  constructor(
    /**
     * The call stack item currently being processed.
     */
    public callStackItem: PrivateCallStackItem,
    /**
     * Other private call stack items to be processed.
     */
    public privateCallStackPreimages: Tuple<PrivateCallStackItem, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * The proof of the execution of this private call.
     */
    public proof: Proof,
    /**
     * The verification key for the function being invoked.
     */
    public vk: VerificationKey,
    /**
     * The membership witness for the function leaf corresponding to the function being invoked.
     */
    public functionLeafMembershipWitness: MembershipWitness<typeof FUNCTION_TREE_HEIGHT>,
    /**
     * The membership witness for the contract leaf corresponding to the contract on which the function is being
     * invoked.
     */
    public contractLeafMembershipWitness: MembershipWitness<typeof CONTRACT_TREE_HEIGHT>,
    /**
     * The membership witnesses for read requests created by the function being invoked.
     */
    public readRequestMembershipWitnesses: Tuple<ReadRequestMembershipWitness, typeof MAX_READ_REQUESTS_PER_CALL>,
    /**
     * The address of the portal contract corresponding to the contract on which the function is being invoked.
     */
    public portalContractAddress: Fr,
    /**
     * The hash of the ACIR of the function being invoked.
     */
    public acirHash: Fr,
  ) {
    assertMemberLength(this, 'privateCallStackPreimages', MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL);
    assertMemberLength(this, 'readRequestMembershipWitnesses', MAX_READ_REQUESTS_PER_CALL);
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PrivateCallData>) {
    return [
      // NOTE: Must have same order as CPP.
      fields.callStackItem,
      fields.privateCallStackPreimages,
      fields.proof,
      fields.vk,
      fields.functionLeafMembershipWitness,
      fields.contractLeafMembershipWitness,
      fields.readRequestMembershipWitnesses,
      fields.portalContractAddress,
      fields.acirHash,
    ] as const;
  }

  static from(fields: FieldsOf<PrivateCallData>): PrivateCallData {
    return new PrivateCallData(...PrivateCallData.getFields(fields));
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(...PrivateCallData.getFields(this));
  }
}

/**
 * Input to the private kernel circuit - initial call.
 */
export class PrivateKernelInputsInit {
  constructor(
    /**
     * The transaction request which led to the creation of these inputs.
     */
    public txRequest: TxRequest,
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
    return serializeToBuffer(this.txRequest, this.privateCall);
  }
}

/**
 * Input to the private kernel circuit - Inner call.
 */
export class PrivateKernelInputsInner {
  constructor(
    /**
     * The previous kernel data (dummy if this is the first kernel).
     */
    public previousKernel: PreviousKernelData,
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
}

/**
 * Input to the private kernel circuit - Final ordering call.
 */
export class PrivateKernelInputsOrdering {
  constructor(
    /**
     * The previous kernel data
     */
    public previousKernel: PreviousKernelData,
    /**
     * Contains hints for the transient read requests to localize corresponding commitments.
     */
    public readCommitmentHints: Tuple<Fr, typeof MAX_READ_REQUESTS_PER_TX>,
    /**
     * Contains hints for the transient nullifiers to localize corresponding commitments.
     */
    public nullifierCommitmentHints: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.readCommitmentHints);
  }
}
