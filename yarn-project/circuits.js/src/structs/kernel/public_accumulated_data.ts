import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import {
  type MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { PublicDataUpdateRequest } from '../public_data_update_request.js';
import { SideEffect, SideEffectLinkedToNoteHash } from '../side_effects.js';

export class PublicAccumulatedData {
  constructor(
    /**
     * The new note hashes made in this transaction.
     */
    public newNoteHashes: Tuple<SideEffect, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public newNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * Accumulated encrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
     */
    public encryptedLogsHash: Fr,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Represented as a tuple of 2 fields in order to fit in all of the 256 bits of sha256 hash.
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
    /**
     * Current public call stack.
     */
    public publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.newNoteHashes,
      this.newNullifiers,
      this.newL2ToL1Msgs,
      this.encryptedLogsHash,
      this.unencryptedLogsHash,
      this.encryptedLogPreimagesLength,
      this.unencryptedLogPreimagesLength,
      this.publicDataUpdateRequests,
      this.publicCallStack,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  isEmpty(): boolean {
    return (
      this.newNoteHashes.every(x => x.isEmpty()) &&
      this.newNullifiers.every(x => x.isEmpty()) &&
      this.newL2ToL1Msgs.every(x => x.isZero()) &&
      this.encryptedLogsHash.isZero() &&
      this.unencryptedLogsHash.isZero() &&
      this.encryptedLogPreimagesLength.isZero() &&
      this.unencryptedLogPreimagesLength.isZero() &&
      this.publicDataUpdateRequests.every(x => x.isEmpty()) &&
      this.publicCallStack.every(x => x.isEmpty())
    );
  }

  [inspect.custom]() {
    // print out the non-empty fields
    return `PublicAccumulatedData {
  newNoteHashes: [${this.newNoteHashes.map(h => h.toString()).join(', ')}],
  newNullifiers: [${this.newNullifiers.map(h => h.toString()).join(', ')}],
  newL2ToL1Msgs: [${this.newL2ToL1Msgs.map(h => h.toString()).join(', ')}],
  encryptedLogsHash: [${this.encryptedLogsHash}],
  unencryptedLogsHash: [${this.unencryptedLogsHash}],
  encryptedLogPreimagesLength: ${this.encryptedLogPreimagesLength}
  unencryptedLogPreimagesLength: ${this.unencryptedLogPreimagesLength}
  publicDataUpdateRequests: [${this.publicDataUpdateRequests.map(h => h.toString()).join(', ')}],
  publicCallStack: [${this.publicCallStack.map(h => h.toString()).join(', ')}],
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
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, SideEffect),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
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
      makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, SideEffect.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
    );
  }
}
