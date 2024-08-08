import { type FieldsOf, makeTuple } from '@aztec/foundation/array';
import { arraySerializedSizeOfNonEmpty } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_L2_TO_L1_MSGS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { Gas } from '../gas.js';
import { ScopedL2ToL1Message } from '../l2_to_l1_message.js';
import { LogHash, ScopedLogHash } from '../log_hash.js';
import { PublicDataUpdateRequest } from '../public_data_update_request.js';

/**
 * Data that is accumulated during the execution of the transaction.
 */
export class CombinedAccumulatedData {
  constructor(
    /**
     * The new note hashes made in this transaction.
     */
    public noteHashes: Tuple<Fr, typeof MAX_NOTE_HASHES_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public nullifiers: Tuple<Fr, typeof MAX_NULLIFIERS_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public l2ToL1Msgs: Tuple<ScopedL2ToL1Message, typeof MAX_L2_TO_L1_MSGS_PER_TX>,
    /**
     * Accumulated note logs hashes from all the previous kernel iterations.
     */
    public noteEncryptedLogsHashes: Tuple<LogHash, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    /**
     * Accumulated encrypted logs hashes from all the previous kernel iterations.
     */
    public encryptedLogsHashes: Tuple<ScopedLogHash, typeof MAX_ENCRYPTED_LOGS_PER_TX>,
    /**
     * Accumulated unencrypted logs hash from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public unencryptedLogsHashes: Tuple<ScopedLogHash, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
    /**
     * Total accumulated length of the encrypted note log preimages emitted in all the previous kernel iterations
     */
    public noteEncryptedLogPreimagesLength: Fr,
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

    /** Gas used during this transaction */
    public gasUsed: Gas,
  ) {}

  getSize() {
    return (
      arraySerializedSizeOfNonEmpty(this.noteHashes) +
      arraySerializedSizeOfNonEmpty(this.nullifiers) +
      arraySerializedSizeOfNonEmpty(this.l2ToL1Msgs) +
      arraySerializedSizeOfNonEmpty(this.noteEncryptedLogsHashes) +
      arraySerializedSizeOfNonEmpty(this.encryptedLogsHashes) +
      arraySerializedSizeOfNonEmpty(this.unencryptedLogsHashes) +
      this.noteEncryptedLogPreimagesLength.size +
      this.encryptedLogPreimagesLength.size +
      this.unencryptedLogPreimagesLength.size +
      arraySerializedSizeOfNonEmpty(this.publicDataUpdateRequests) +
      this.gasUsed.toBuffer().length
    );
  }

  static getFields(fields: FieldsOf<CombinedAccumulatedData>) {
    return [
      fields.noteHashes,
      fields.nullifiers,
      fields.l2ToL1Msgs,
      fields.noteEncryptedLogsHashes,
      fields.encryptedLogsHashes,
      fields.unencryptedLogsHashes,
      fields.noteEncryptedLogPreimagesLength,
      fields.encryptedLogPreimagesLength,
      fields.unencryptedLogPreimagesLength,
      fields.publicDataUpdateRequests,
      fields.gasUsed,
    ] as const;
  }

  static from(fields: FieldsOf<CombinedAccumulatedData>): CombinedAccumulatedData {
    return new CombinedAccumulatedData(...CombinedAccumulatedData.getFields(fields));
  }

  toBuffer() {
    return serializeToBuffer(...CombinedAccumulatedData.getFields(this));
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
      reader.readArray(MAX_NOTE_HASHES_PER_TX, Fr),
      reader.readArray(MAX_NULLIFIERS_PER_TX, Fr),
      reader.readArray(MAX_L2_TO_L1_MSGS_PER_TX, ScopedL2ToL1Message),
      reader.readArray(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, LogHash),
      reader.readArray(MAX_ENCRYPTED_LOGS_PER_TX, ScopedLogHash),
      reader.readArray(MAX_UNENCRYPTED_LOGS_PER_TX, ScopedLogHash),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest),
      reader.readObject(Gas),
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
      makeTuple(MAX_NOTE_HASHES_PER_TX, Fr.zero),
      makeTuple(MAX_NULLIFIERS_PER_TX, Fr.zero),
      makeTuple(MAX_L2_TO_L1_MSGS_PER_TX, ScopedL2ToL1Message.empty),
      makeTuple(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, LogHash.empty),
      makeTuple(MAX_ENCRYPTED_LOGS_PER_TX, ScopedLogHash.empty),
      makeTuple(MAX_UNENCRYPTED_LOGS_PER_TX, ScopedLogHash.empty),
      Fr.zero(),
      Fr.zero(),
      Fr.zero(),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataUpdateRequest.empty),
      Gas.empty(),
    );
  }

  [inspect.custom]() {
    return `CombinedAccumulatedData {
      noteHashes: [${this.noteHashes
        .filter(x => !x.isZero())
        .map(x => inspect(x))
        .join(', ')}],
      nullifiers: [${this.nullifiers
        .filter(x => !x.isZero())
        .map(x => inspect(x))
        .join(', ')}],
      l2ToL1Msgs: [${this.l2ToL1Msgs
        .filter(x => !x.isEmpty())
        .map(x => inspect(x))
        .join(', ')}],
      noteEncryptedLogsHash:  [${this.noteEncryptedLogsHashes
        .filter(x => !x.isEmpty())
        .map(x => inspect(x))
        .join(', ')}]
      encryptedLogsHash: [${this.encryptedLogsHashes
        .filter(x => !x.isEmpty())
        .map(x => inspect(x))
        .join(', ')}]
      unencryptedLogsHashes: : [${this.unencryptedLogsHashes
        .filter(x => !x.isEmpty())
        .map(x => inspect(x))
        .join(', ')}],
      noteEncryptedLogPreimagesLength: ${this.noteEncryptedLogPreimagesLength.toString()},
      encryptedLogPreimagesLength: ${this.encryptedLogPreimagesLength.toString()},
      unencryptedLogPreimagesLength: ${this.unencryptedLogPreimagesLength.toString()},
      publicDataUpdateRequests: [${this.publicDataUpdateRequests
        .filter(x => !x.isEmpty())
        .map(x => inspect(x))
        .join(', ')}],
      gasUsed: ${inspect(this.gasUsed)}
    }`;
  }
}
