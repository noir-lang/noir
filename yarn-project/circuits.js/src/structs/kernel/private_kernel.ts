import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import {
  CONTRACT_TREE_HEIGHT,
  FUNCTION_TREE_HEIGHT,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_TX,
} from '../../constants.gen.js';
import { GrumpkinPrivateKey } from '../../types/grumpkin_private_key.js';
import { CallRequest } from '../call_request.js';
import { PrivateCallStackItem } from '../call_stack_item.js';
import { MembershipWitness } from '../membership_witness.js';
import { Proof } from '../proof.js';
import { ReadRequestMembershipWitness } from '../read_request_membership_witness.js';
import { SideEffect, SideEffectLinkedToNoteHash } from '../side_effects.js';
import { TxRequest } from '../tx_request.js';
import { VerificationKey } from '../verification_key.js';
import { PreviousKernelData } from './previous_kernel_data.js';

/**
 * Private call data.
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
    public privateCallStack: Tuple<CallRequest, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    /**
     * Other public call stack items to be processed.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
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
  ) {}

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PrivateCallData>) {
    return [
      fields.callStackItem,
      fields.privateCallStack,
      fields.publicCallStack,
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

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateCallData {
    const reader = BufferReader.asReader(buffer);
    return new PrivateCallData(
      reader.readObject(PrivateCallStackItem),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, CallRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, CallRequest),
      reader.readObject(Proof),
      reader.readObject(VerificationKey),
      reader.readObject(MembershipWitness.deserializer(FUNCTION_TREE_HEIGHT)),
      reader.readObject(MembershipWitness.deserializer(CONTRACT_TREE_HEIGHT)),
      reader.readArray(MAX_READ_REQUESTS_PER_CALL, ReadRequestMembershipWitness),
      reader.readObject(Fr),
      reader.readObject(Fr),
    );
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

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelInputsInit {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInputsInit(reader.readObject(TxRequest), reader.readObject(PrivateCallData));
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

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelInputsInner {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInputsInner(reader.readObject(PreviousKernelData), reader.readObject(PrivateCallData));
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
     * The sorted new commitments.
     */
    public sortedNewCommitments: Tuple<SideEffect, typeof MAX_NEW_COMMITMENTS_PER_TX>,
    /**
     * The sorted new commitments indexes. Maps original to sorted.
     */
    public sortedNewCommitmentsIndexes: Tuple<number, typeof MAX_NEW_COMMITMENTS_PER_TX>,
    /**
     * Contains hints for the transient read requests to localize corresponding commitments.
     */
    public readCommitmentHints: Tuple<Fr, typeof MAX_READ_REQUESTS_PER_TX>,
    /**
     * The sorted new nullifiers. Maps original to sorted.
     */
    public sortedNewNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The sorted new nullifiers indexes.
     */
    public sortedNewNullifiersIndexes: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Contains hints for the transient nullifiers to localize corresponding commitments.
     */
    public nullifierCommitmentHints: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The master nullifier secret keys for the nullifier key validation requests.
     */
    public masterNullifierSecretKeys: Tuple<GrumpkinPrivateKey, typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.previousKernel,
      this.sortedNewCommitments,
      this.sortedNewCommitmentsIndexes,
      this.readCommitmentHints,
      this.sortedNewNullifiers,
      this.sortedNewNullifiersIndexes,
      this.nullifierCommitmentHints,
      this.masterNullifierSecretKeys,
    );
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelInputsOrdering {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelInputsOrdering(
      reader.readObject(PreviousKernelData),
      reader.readArray(MAX_NEW_COMMITMENTS_PER_TX, SideEffect),
      reader.readNumbers(MAX_NEW_COMMITMENTS_PER_TX),
      reader.readArray(MAX_READ_REQUESTS_PER_TX, Fr),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readNumbers(MAX_NEW_NULLIFIERS_PER_TX),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Fr),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar),
    );
  }
}
