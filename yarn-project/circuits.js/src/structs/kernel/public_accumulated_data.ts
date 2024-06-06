import { makeTuple } from '@aztec/foundation/array';
import { arraySerializedSizeOfNonEmpty } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  type MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { Gas } from '../gas.js';
import { LogHash } from '../log_hash.js';
import { NoteHash } from '../note_hash.js';
import { Nullifier } from '../nullifier.js';
import { PublicDataUpdateRequest } from '../public_data_update_request.js';

export class PublicAccumulatedData {
  constructor(
    /**
     * The new note hashes made in this transaction.
     */
    public readonly newNoteHashes: Tuple<NoteHash, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public readonly newNullifiers: Tuple<Nullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public readonly newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,
    /**
     * Accumulated encrypted note logs hashes from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public readonly noteEncryptedLogsHashes: Tuple<LogHash, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    /**
     * Accumulated encrypted logs hashes from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public readonly encryptedLogsHashes: Tuple<LogHash, typeof MAX_ENCRYPTED_LOGS_PER_TX>,
    /**
     * Accumulated unencrypted logs hashes from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public readonly unencryptedLogsHashes: Tuple<LogHash, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
    /**
     * All the public data update requests made in this transaction.
     */
    public readonly publicDataUpdateRequests: Tuple<
      PublicDataUpdateRequest,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
    /**
     * Current public call stack.
     */
    public readonly publicCallStack: Tuple<CallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,

    /** Gas used so far by the transaction. */
    public readonly gasUsed: Gas,
  ) {}

  getSize() {
    return (
      arraySerializedSizeOfNonEmpty(this.newNoteHashes) +
      arraySerializedSizeOfNonEmpty(this.newNullifiers) +
      arraySerializedSizeOfNonEmpty(this.newL2ToL1Msgs) +
      arraySerializedSizeOfNonEmpty(this.noteEncryptedLogsHashes) +
      arraySerializedSizeOfNonEmpty(this.encryptedLogsHashes) +
      arraySerializedSizeOfNonEmpty(this.unencryptedLogsHashes) +
      arraySerializedSizeOfNonEmpty(this.publicDataUpdateRequests) +
      arraySerializedSizeOfNonEmpty(this.publicCallStack) +
      this.gasUsed.toBuffer().length
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.newNoteHashes,
      this.newNullifiers,
      this.newL2ToL1Msgs,
      this.noteEncryptedLogsHashes,
      this.encryptedLogsHashes,
      this.unencryptedLogsHashes,
      this.publicDataUpdateRequests,
      this.publicCallStack,
      this.gasUsed,
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
      this.noteEncryptedLogsHashes.every(x => x.isEmpty()) &&
      this.encryptedLogsHashes.every(x => x.isEmpty()) &&
      this.unencryptedLogsHashes.every(x => x.isEmpty()) &&
      this.publicDataUpdateRequests.every(x => x.isEmpty()) &&
      this.publicCallStack.every(x => x.isEmpty()) &&
      this.gasUsed.isEmpty()
    );
  }

  [inspect.custom]() {
    // print out the non-empty fields
    return `PublicAccumulatedData {
  newNoteHashes: [${this.newNoteHashes
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  newNullifiers: [${this.newNullifiers
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  newL2ToL1Msgs: [${this.newL2ToL1Msgs
    .filter(x => !x.isZero())
    .map(h => inspect(h))
    .join(', ')}],
  noteEncryptedLogsHashes: [${this.noteEncryptedLogsHashes
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  encryptedLogsHashes: [${this.encryptedLogsHashes
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  unencryptedLogsHashes: [${this.unencryptedLogsHashes
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  publicDataUpdateRequests: [${this.publicDataUpdateRequests
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  publicCallStack: [${this.publicCallStack
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  gasUsed: [${inspect(this.gasUsed)}]
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
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, NoteHash),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Nullifier),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, LogHash),
      reader.readArray(MAX_ENCRYPTED_LOGS_PER_TX, LogHash),
      reader.readArray(MAX_UNENCRYPTED_LOGS_PER_TX, LogHash),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readObject(Gas),
    );
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new this(
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, NoteHash),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Nullifier),
      reader.readFieldArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX),
      reader.readArray(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, LogHash),
      reader.readArray(MAX_ENCRYPTED_LOGS_PER_TX, LogHash),
      reader.readArray(MAX_UNENCRYPTED_LOGS_PER_TX, LogHash),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest),
      reader.readObject(Gas),
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
      makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, NoteHash.empty),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Nullifier.empty),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, LogHash.empty),
      makeTuple(MAX_ENCRYPTED_LOGS_PER_TX, LogHash.empty),
      makeTuple(MAX_UNENCRYPTED_LOGS_PER_TX, LogHash.empty),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty),
      Gas.empty(),
    );
  }
}
