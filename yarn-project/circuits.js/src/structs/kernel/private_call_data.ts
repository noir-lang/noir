import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import {
  FUNCTION_TREE_HEIGHT,
  MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
} from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { MembershipWitness } from '../membership_witness.js';
import { NoteHashReadRequestMembershipWitness } from '../note_hash_read_request_membership_witness.js';
import { PrivateCallStackItem } from '../private_call_stack_item.js';
import { Proof } from '../proof.js';
import { VerificationKey } from '../verification_key.js';

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
     * Artifact hash of the contract class for this private call.
     */
    public contractClassArtifactHash: Fr,
    /**
     * Public bytecode commitment for the contract class for this private call.
     */
    public contractClassPublicBytecodeCommitment: Fr,
    /**
     * Public keys hash of the contract instance.
     */
    public publicKeysHash: Fr,
    /**
     * Salted initialization hash of the contract instance.
     */
    public saltedInitializationHash: Fr,
    /**
     * The membership witness for the function leaf corresponding to the function being invoked.
     */
    public functionLeafMembershipWitness: MembershipWitness<typeof FUNCTION_TREE_HEIGHT>,
    /**
     * The membership witnesses for read requests created by the function being invoked.
     */
    public noteHashReadRequestMembershipWitnesses: Tuple<
      NoteHashReadRequestMembershipWitness,
      typeof MAX_NOTE_HASH_READ_REQUESTS_PER_CALL
    >,
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
      fields.contractClassArtifactHash,
      fields.contractClassPublicBytecodeCommitment,
      fields.publicKeysHash,
      fields.saltedInitializationHash,
      fields.functionLeafMembershipWitness,
      fields.noteHashReadRequestMembershipWitnesses,
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
      reader.readObject(Fr),
      reader.readObject(Fr),
      reader.readObject(Fr),
      reader.readObject(Fr),
      reader.readObject(MembershipWitness.deserializer(FUNCTION_TREE_HEIGHT)),
      reader.readArray(MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, NoteHashReadRequestMembershipWitness),
      reader.readObject(Fr),
      reader.readObject(Fr),
    );
  }
}
