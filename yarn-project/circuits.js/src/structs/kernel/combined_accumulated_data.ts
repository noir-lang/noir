import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { createDebugOnlyLogger } from '@aztec/foundation/log';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import {
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX,
  MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_REVERTIBLE_NOTE_HASHES_PER_TX,
  MAX_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { PublicDataUpdateRequest } from '../public_data_update_request.js';
import { RevertCode } from '../revert_code.js';
import { SideEffect, SideEffectLinkedToNoteHash, sideEffectCmp } from '../side_effects.js';

const log = createDebugOnlyLogger('aztec:combined_accumulated_data');

/**
 * Data that is accumulated during the execution of the transaction.
 */
export class CombinedAccumulatedData {
  constructor(
    /**
     * Flag indicating whether the transaction reverted.
     */
    public revertCode: RevertCode,
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
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public encryptedLogsHash: Fr,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public unencryptedLogsHash: Fr,
    /**
     * Total accumulated length of the encrypted log preimages emitted in all the previous kernel iterations
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Total accumulated length of the unencrypted log preimages emitted in all the previous kernel iterations
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.revertCode,
      this.newNoteHashes,
      this.newNullifiers,
      this.privateCallStack,
      this.publicCallStack,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.publicDataUpdateRequests,
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
      RevertCode.fromBuffer(reader),
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
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
      RevertCode.OK,
      makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
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
    if (!nonRevertible.revertCode.isOK() && !revertible.isEmpty()) {
      log(inspect(revertible));
      throw new Error('Revertible data should be empty if the transaction is revertCode');
    }

    const newNoteHashes = padArrayEnd(
      [...nonRevertible.newNoteHashes, ...revertible.newNoteHashes].filter(x => !x.isEmpty()).sort(sideEffectCmp),
      SideEffect.empty(),
      MAX_NEW_NOTE_HASHES_PER_TX,
    );

    const newNullifiers = padArrayEnd(
      [...nonRevertible.newNullifiers, ...revertible.newNullifiers].filter(x => !x.isEmpty()).sort(sideEffectCmp),
      SideEffectLinkedToNoteHash.empty(),
      MAX_NEW_NULLIFIERS_PER_TX,
    );

    const publicCallStack = padArrayEnd(
      [...nonRevertible.publicCallStack, ...revertible.publicCallStack].filter(x => !x.isEmpty()),
      CallRequest.empty(),
      MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    );

    const nonSquashedWrites = [
      ...revertible.publicDataUpdateRequests,
      ...nonRevertible.publicDataUpdateRequests,
    ].filter(x => !x.isEmpty());

    const squashedWrites = Array.from(
      nonSquashedWrites
        .reduce<Map<string, PublicDataUpdateRequest>>((acc, curr) => {
          acc.set(curr.leafSlot.toString(), curr);
          return acc;
        }, new Map())
        .values(),
    );

    const publicDataUpdateRequests = padArrayEnd(
      squashedWrites,
      PublicDataUpdateRequest.empty(),
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    );

    return new CombinedAccumulatedData(
      nonRevertible.revertCode,
      newNoteHashes,
      newNullifiers,
      revertible.privateCallStack,
      publicCallStack,
      revertible.newL2ToL1Msgs,
      revertible.encryptedLogsHash,
      revertible.unencryptedLogsHash,
      revertible.encryptedLogPreimagesLength,
      revertible.unencryptedLogPreimagesLength,
      publicDataUpdateRequests,
    );
  }
}

export class PublicAccumulatedRevertibleData {
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
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public encryptedLogsHash: Fr,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public unencryptedLogsHash: Fr,
    /**
     * Total accumulated length of the encrypted log preimages emitted in all the previous kernel iterations
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Total accumulated length of the unencrypted log preimages emitted in all the previous kernel iterations
     */
    public unencryptedLogPreimagesLength: Fr,
    /**
     * All the public data update requests made in this transaction.
     */
    public publicDataUpdateRequests: Tuple<
      PublicDataUpdateRequest,
      typeof MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
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
      this.publicDataUpdateRequests,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  isEmpty(): boolean {
    return (
      this.newNoteHashes.every(x => x.isEmpty()) &&
      this.newNullifiers.every(x => x.isEmpty()) &&
      this.privateCallStack.every(x => x.isEmpty()) &&
      this.publicCallStack.every(x => x.isEmpty()) &&
      this.newL2ToL1Msgs.every(x => x.isZero()) &&
      this.encryptedLogsHash.isZero() &&
      this.unencryptedLogsHash.isZero() &&
      this.encryptedLogPreimagesLength.isZero() &&
      this.unencryptedLogPreimagesLength.isZero() &&
      this.publicDataUpdateRequests.every(x => x.isEmpty())
    );
  }

  [inspect.custom]() {
    // print out the non-empty fields
    return `PublicAccumulatedRevertibleData {
  newNoteHashes: [${this.newNoteHashes.map(h => h.toString()).join(', ')}],
  newNullifiers: [${this.newNullifiers.map(h => h.toString()).join(', ')}],
  privateCallStack: [${this.privateCallStack.map(h => h.toString()).join(', ')}],
  publicCallStack: [${this.publicCallStack.map(h => h.toString()).join(', ')}],
  newL2ToL1Msgs: [${this.newL2ToL1Msgs.map(h => h.toString()).join(', ')}],
  encryptedLogsHash: ${this.encryptedLogsHash},
  unencryptedLogsHash: ${this.unencryptedLogsHash},
  encryptedLogPreimagesLength: ${this.encryptedLogPreimagesLength}
  unencryptedLogPreimagesLength: ${this.unencryptedLogPreimagesLength}
  publicDataUpdateRequests: [${this.publicDataUpdateRequests.map(h => h.toString()).join(', ')}],
}`;
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(
      reader.readArray(MAX_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
    );
  }

  static fromPrivateAccumulatedRevertibleData(finalData: PrivateAccumulatedRevertibleData) {
    return new this(
      padArrayEnd(finalData.newNoteHashes, SideEffect.empty(), MAX_REVERTIBLE_NOTE_HASHES_PER_TX),
      padArrayEnd(finalData.newNullifiers, SideEffectLinkedToNoteHash.empty(), MAX_REVERTIBLE_NULLIFIERS_PER_TX),
      finalData.privateCallStack,
      padArrayEnd(finalData.publicCallStack, CallRequest.empty(), MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX),
      finalData.newL2ToL1Msgs,
      finalData.encryptedLogsHash,
      finalData.unencryptedLogsHash,
      finalData.encryptedLogPreimagesLength,
      finalData.unencryptedLogPreimagesLength,
      makeTuple(MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
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
      makeTuple(MAX_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
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
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public encryptedLogsHash: Fr,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public unencryptedLogsHash: Fr,
    /**
     * Total accumulated length of the encrypted log preimages emitted in all the previous kernel iterations
     */
    public encryptedLogPreimagesLength: Fr,
    /**
     * Total accumulated length of the unencrypted log preimages emitted in all the previous kernel iterations
     */
    public unencryptedLogPreimagesLength: Fr,
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
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
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
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
    );
  }
}

export class PrivateAccumulatedNonRevertibleData {
  constructor(
    /**
     * Flag indicating whether the transaction reverted.
     */
    public revertCode: RevertCode,
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
    return serializeToBuffer(this.revertCode.toBuffer(), this.newNoteHashes, this.newNullifiers, this.publicCallStack);
  }

  static fromBuffer(buffer: Buffer | BufferReader): PrivateAccumulatedNonRevertibleData {
    const reader = BufferReader.asReader(buffer);
    return new PrivateAccumulatedNonRevertibleData(
      RevertCode.fromBuffer(reader),
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
      RevertCode.OK,
      makeTuple(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
    );
  }
}

export class PublicAccumulatedNonRevertibleData {
  constructor(
    /**
     * Flag indicating whether the transaction reverted.
     */
    public revertCode: RevertCode,
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
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.revertCode,
      this.newNoteHashes,
      this.newNullifiers,
      this.publicCallStack,
      this.publicDataUpdateRequests,
    );
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(
      RevertCode.fromBuffer(reader),
      reader.readArray(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readArray(MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
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
      RevertCode.OK,
      makeTuple(MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
    );
  }

  static fromPrivateAccumulatedNonRevertibleData(data: PrivateAccumulatedNonRevertibleData) {
    return new this(
      data.revertCode,
      data.newNoteHashes,
      data.newNullifiers,
      data.publicCallStack,
      makeTuple(MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
    );
  }

  [inspect.custom]() {
    return `PublicAccumulatedNonRevertibleData {
  revertCode: ${this.revertCode},
  newNoteHashes: [${this.newNoteHashes.map(h => h.toString()).join(', ')}],
  newNullifiers: [${this.newNullifiers.map(h => h.toString()).join(', ')}],
  publicCallStack: [${this.publicCallStack.map(h => h.toString()).join(', ')}],
  publicDataUpdateRequests: [${this.publicDataUpdateRequests.map(h => h.toString()).join(', ')}],
}`;
  }
}
