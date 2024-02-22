import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX,
  MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX,
  MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_READ_REQUESTS_PER_TX,
  MAX_REVERTIBLE_NOTE_HASHES_PER_TX,
  MAX_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX,
  MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NUM_FIELDS_PER_SHA256,
} from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { NullifierKeyValidationRequestContext } from '../nullifier_key_validation_request.js';
import { SideEffect, SideEffectLinkedToNoteHash } from '../side_effects.js';
import { NewContractData } from './new_contract_data.js';

/**
 * Read operations from the public state tree.
 */
export class PublicDataRead {
  constructor(
    /**
     * Index of the leaf in the public data tree.
     */
    public readonly leafSlot: Fr,
    /**
     * Returned value from the public data tree.
     */
    public readonly value: Fr,
    /**
     * Optional side effect counter tracking position of this event in tx execution.
     */
    public readonly sideEffectCounter?: number,
  ) {}

  static from(args: {
    /**
     * Index of the leaf in the public data tree.
     */
    leafIndex: Fr;
    /**
     * Returned value from the public data tree.
     */
    value: Fr;
  }) {
    return new PublicDataRead(args.leafIndex, args.value);
  }

  toBuffer() {
    return serializeToBuffer(this.leafSlot, this.value);
  }

  isEmpty() {
    return this.leafSlot.isZero() && this.value.isZero();
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataRead(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  static empty() {
    return new PublicDataRead(Fr.ZERO, Fr.ZERO);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafSlot.toFriendlyJSON()}: ${this.value.toFriendlyJSON()}`;
  }
}

/**
 * Write operations on the public data tree including the previous value.
 */
export class PublicDataUpdateRequest {
  constructor(
    /**
     * Index of the leaf in the public data tree which is to be updated.
     */
    public readonly leafSlot: Fr,
    /**
     * New value of the leaf.
     */
    public readonly newValue: Fr,
    /**
     * Optional side effect counter tracking position of this event in tx execution.
     */
    public readonly sideEffectCounter?: number,
  ) {}

  static from(args: {
    /**
     * Index of the leaf in the public data tree which is to be updated.
     */
    leafIndex: Fr;
    /**
     * New value of the leaf.
     */
    newValue: Fr;
  }) {
    return new PublicDataUpdateRequest(args.leafIndex, args.newValue);
  }

  toBuffer() {
    return serializeToBuffer(this.leafSlot, this.newValue);
  }

  isEmpty() {
    return this.leafSlot.isZero() && this.newValue.isZero();
  }

  equals(other: PublicDataUpdateRequest) {
    return this.leafSlot.equals(other.leafSlot) && this.newValue.equals(other.newValue);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataUpdateRequest(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  static empty() {
    return new PublicDataUpdateRequest(Fr.ZERO, Fr.ZERO);
  }

  toFriendlyJSON() {
    return `Leaf=${this.leafSlot.toFriendlyJSON()}: ${this.newValue.toFriendlyJSON()}`;
  }
}

/**
 * Data that is accumulated during the execution of the transaction.
 */
export class CombinedAccumulatedData {
  constructor(
    /**
     * All the read requests made in this transaction.
     */
    public readRequests: Tuple<SideEffect, typeof MAX_READ_REQUESTS_PER_TX>,
    /**
     * All the nullifier key validation requests made in this transaction.
     */
    public nullifierKeyValidationRequests: Tuple<
      NullifierKeyValidationRequestContext,
      typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
    >,
    /**
     * The new note hashes made in this transaction.
     */
    public newNoteHashes: Tuple<SideEffect, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Current private call stack.
     */
    public privateCallStack: Tuple<CallRequest, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * Accumulated encrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public encryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Total accumulated length of the encrypted log preimages emitted in all the previous kernel iterations
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Total accumulated length of the unencrypted log preimages emitted in all the previous kernel iterations
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * All the new contracts deployed in this transaction.
     */
    public newContracts: Tuple<NewContractData, typeof MAX_NEW_CONTRACTS_PER_TX>,
    /**
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * All the public data reads made in this transaction.
     */
    public publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.readRequests,
      this.nullifierKeyValidationRequests,
      this.newNoteHashes,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.newContracts,
      this.publicDataUpdateRequests,
      this.publicDataReads,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedAccumulatedData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedAccumulatedData(
      reader.readArray(MAX_READ_REQUESTS_PER_TX, SideEffect),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext),
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(2, Fr),
      reader.readArray(2, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, NewContractData),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return CombinedAccumulatedData.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new CombinedAccumulatedData(
      makeTuple(MAX_READ_REQUESTS_PER_TX, SideEffect.empty),
      makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext.empty),
      makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(2, Fr.zero),
      makeTuple(2, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, NewContractData.empty),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }

  /**
   *
   * @param nonRevertible the non-revertible accumulated data
   * @param revertible the revertible accumulated data
   * @returns a new CombinedAccumulatedData object, squashing the two inputs, and condensing arrays
   */
  public static recombine(
    nonRevertible: PublicAccumulatedNonRevertibleData,
    revertible: PublicAccumulatedRevertibleData,
  ): CombinedAccumulatedData {
    const newNoteHashes = padArrayEnd(
      [...nonRevertible.newNoteHashes, ...revertible.newNoteHashes].filter(x => !x.isEmpty()),
      SideEffect.empty(),
      MAX_NEW_NOTE_HASHES_PER_TX,
    );

    const newNullifiers = padArrayEnd(
      [...nonRevertible.newNullifiers, ...revertible.newNullifiers].filter(x => !x.isEmpty()),
      SideEffectLinkedToNoteHash.empty(),
      MAX_NEW_NULLIFIERS_PER_TX,
    );

    const publicCallStack = padArrayEnd(
      [...nonRevertible.publicCallStack, ...revertible.publicCallStack].filter(x => !x.isEmpty()),
      CallRequest.empty(),
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    );

    const publicDataUpdateRequests = padArrayEnd(
      [...nonRevertible.publicDataUpdateRequests, ...revertible.publicDataUpdateRequests].filter(x => !x.isEmpty()),
      PublicDataUpdateRequest.empty(),
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    );

    const publicDataReads = padArrayEnd(
      [...nonRevertible.publicDataReads, ...revertible.publicDataReads].filter(x => !x.isEmpty()),
      PublicDataRead.empty(),
      MAX_PUBLIC_DATA_READS_PER_TX,
    );

    return new CombinedAccumulatedData(
      revertible.readRequests,
      revertible.nullifierKeyValidationRequests,
      newNoteHashes,
      newNullifiers,
      revertible.privateCallStack,
      publicCallStack,
      revertible.newL2ToL1Msgs,
      revertible.encryptedLogsHash,
      revertible.unencryptedLogsHash,
      revertible.encryptedLogPreimagesLength,
      revertible.unencryptedLogPreimagesLength,
      revertible.newContracts,
      publicDataUpdateRequests,
      publicDataReads,
    );
  }
}

export class PublicAccumulatedRevertibleData {
  constructor(
    /**
     * All the read requests made in this transaction.
     */
    public readRequests: Tuple<SideEffect, typeof MAX_READ_REQUESTS_PER_TX>,
    /**
     * All the nullifier key validation requests made in this transaction.
     */
    public nullifierKeyValidationRequests: Tuple<
      NullifierKeyValidationRequestContext,
      typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
    >,
    /**
     * The new note hashes made in this transaction.
     */
    public newNoteHashes: Tuple<SideEffect, typeof MAX_REVERTIBLE_NOTE_HASHES_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_REVERTIBLE_NULLIFIERS_PER_TX>,
    /**
     * Current private call stack.
     */
    public privateCallStack: Tuple<CallRequest, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * Accumulated encrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public encryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Total accumulated length of the encrypted log preimages emitted in all the previous kernel iterations
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Total accumulated length of the unencrypted log preimages emitted in all the previous kernel iterations
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * All the new contracts deployed in this transaction.
     */
    public newContracts: Tuple<NewContractData, typeof MAX_NEW_CONTRACTS_PER_TX>,
    /**
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: Tuple<
      PublicDataUpdateRequest,
      typeof MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
    /**
     * All the public data reads made in this transaction.
     */
    public publicDataReads: Tuple<PublicDataRead, typeof MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.readRequests,
      this.nullifierKeyValidationRequests,
      this.newNoteHashes,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.newContracts,
      this.publicDataUpdateRequests,
      this.publicDataReads,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readArray(MAX_READ_REQUESTS_PER_TX, SideEffect),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext),
      reader.readArray(MAX_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(2, Fr),
      reader.readArray(2, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, NewContractData),
      reader.readArray(MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, PublicDataRead),
    );
  }

  static fromPrivateAccumulatedRevertibleData(finalData: PrivateAccumulatedRevertibleData) {
    return new this(
      makeTuple(MAX_READ_REQUESTS_PER_TX, SideEffect.empty),
      makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext.empty),
      padArrayEnd(finalData.newNoteHashes, SideEffect.empty(), MAX_REVERTIBLE_NOTE_HASHES_PER_TX),
      padArrayEnd(finalData.newNullifiers, SideEffectLinkedToNoteHash.empty(), MAX_REVERTIBLE_NULLIFIERS_PER_TX),
      finalData.privateCallStack,
      padArrayEnd(finalData.publicCallStack, CallRequest.empty(), MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX),
      finalData.newL2ToL1Msgs,
      finalData.encryptedLogsHash,
      finalData.unencryptedLogsHash,
      finalData.encryptedLogPreimagesLength,
      finalData.unencryptedLogPreimagesLength,
      finalData.newContracts,
      makeTuple(MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return this.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new this(
      makeTuple(MAX_READ_REQUESTS_PER_TX, SideEffect.empty),
      makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, NullifierKeyValidationRequestContext.empty),
      makeTuple(MAX_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(2, Fr.zero),
      makeTuple(2, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, NewContractData.empty),
      makeTuple(MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }
}

/**
 * Specific accumulated data structure for the final ordering private kernel circuit. It is included
 *  in the final public inputs of private kernel circuit.
 */
export class PrivateAccumulatedRevertibleData {
  constructor(
    /**
     * The new note hashes made in this transaction.
     */
    public newNoteHashes: Tuple<SideEffect, typeof MAX_REVERTIBLE_NOTE_HASHES_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_REVERTIBLE_NULLIFIERS_PER_TX>,
    /**
     * Current private call stack.
     * TODO(#3417): Given this field must empty, should we just remove it?
     */
    public privateCallStack: Tuple<CallRequest, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * Accumulated encrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public encryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public unencryptedLogsHash: Tuple<Fr, typeof NUM_FIELDS_PER_SHA256>,
    /**
     * Total accumulated length of the encrypted log preimages emitted in all the previous kernel iterations
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Total accumulated length of the unencrypted log preimages emitted in all the previous kernel iterations
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * All the new contracts deployed in this transaction.
     */
    public newContracts: Tuple<NewContractData, typeof MAX_NEW_CONTRACTS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.newNoteHashes,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.newContracts,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateAccumulatedRevertibleData {
    const reader = BufferReader.asReader(buffer);
    return new PrivateAccumulatedRevertibleData(
      reader.readArray(MAX_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(2, Fr),
      reader.readArray(2, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, NewContractData),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return PrivateAccumulatedRevertibleData.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new PrivateAccumulatedRevertibleData(
      makeTuple(MAX_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(2, Fr.zero),
      makeTuple(2, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, NewContractData.empty),
    );
  }
}

export class PrivateAccumulatedNonRevertibleData {
  constructor(
    /**
     * The new non-revertible commitments made in this transaction.
     */
    public newNoteHashes: Tuple<SideEffect, typeof MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX>,
    /**
     * The new non-revertible nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX>,
    /**
     * Current public call stack that will produce non-revertible side effects.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.newNoteHashes, this.newNullifiers, this.publicCallStack);
  }

  static fromBuffer(buffer: Buffer | BufferReader): PrivateAccumulatedNonRevertibleData {
    const reader = BufferReader.asReader(buffer);
    return new PrivateAccumulatedNonRevertibleData(
      reader.readArray(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    return PrivateAccumulatedNonRevertibleData.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new PrivateAccumulatedNonRevertibleData(
      makeTuple(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
    );
  }
}

export class PublicAccumulatedNonRevertibleData {
  constructor(
    /**
     * The new non-revertible commitments made in this transaction.
     */
    public newNoteHashes: Tuple<SideEffect, typeof MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX>,
    /**
     * The new non-revertible nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX>,
    /**
     * Current public call stack that will produce non-revertible side effects.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    /**
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: Tuple<
      PublicDataUpdateRequest,
      typeof MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
    /**
     * All the public data reads made in this transaction.
     */
    public publicDataReads: Tuple<PublicDataRead, typeof MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.newNoteHashes, this.newNullifiers, this.publicCallStack);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readArray(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, PublicDataRead),
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    return this.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new this(
      makeTuple(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }

  static fromPrivateAccumulatedNonRevertibleData(data: PrivateAccumulatedNonRevertibleData) {
    return new this(
      data.newNoteHashes,
      data.newNullifiers,
      data.publicCallStack,
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }
}
