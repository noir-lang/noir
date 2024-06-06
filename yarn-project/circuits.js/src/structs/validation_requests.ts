import { makeTuple } from '@aztec/foundation/array';
import { arraySerializedSizeOfNonEmpty } from '@aztec/foundation/collection';
import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import {
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
} from '../constants.gen.js';
import { PublicDataRead } from './public_data_read_request.js';
import { ScopedReadRequest } from './read_request.js';
import { RollupValidationRequests } from './rollup_validation_requests.js';
import { ScopedKeyValidationRequestAndGenerator } from './scoped_key_validation_request_and_generator.js';

/**
 * Validation requests accumulated during the execution of the transaction.
 */
export class ValidationRequests {
  constructor(
    /**
     * Validation requests that cannot be fulfilled in the current context (private or public), and must be instead be
     * forwarded to the rollup for it to take care of them.
     */
    public forRollup: RollupValidationRequests,
    /**
     * All the read requests made in this transaction.
     */
    public noteHashReadRequests: Tuple<ScopedReadRequest, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    /**
     * All the nullifier read requests made in this transaction.
     */
    public nullifierReadRequests: Tuple<ScopedReadRequest, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    /**
     * The nullifier read requests made in this transaction.
     */
    public nullifierNonExistentReadRequests: Tuple<
      ScopedReadRequest,
      typeof MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX
    >,
    /**
     * All the key validation requests made in this transaction.
     */
    public scopedKeyValidationRequestsAndGenerators: Tuple<
      ScopedKeyValidationRequestAndGenerator,
      typeof MAX_KEY_VALIDATION_REQUESTS_PER_TX
    >,
    /**
     * All the public data reads made in this transaction.
     */
    public publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
  ) {}

  getSize() {
    return (
      this.forRollup.getSize() +
      arraySerializedSizeOfNonEmpty(this.noteHashReadRequests) +
      arraySerializedSizeOfNonEmpty(this.nullifierReadRequests) +
      arraySerializedSizeOfNonEmpty(this.nullifierNonExistentReadRequests) +
      arraySerializedSizeOfNonEmpty(this.scopedKeyValidationRequestsAndGenerators) +
      arraySerializedSizeOfNonEmpty(this.publicDataReads)
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.forRollup,
      this.noteHashReadRequests,
      this.nullifierReadRequests,
      this.nullifierNonExistentReadRequests,
      this.scopedKeyValidationRequestsAndGenerators,
      this.publicDataReads,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new ValidationRequests(
      reader.readObject(RollupValidationRequests),
      reader.readArray(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_KEY_VALIDATION_REQUESTS_PER_TX, ScopedKeyValidationRequestAndGenerator),
      reader.readArray(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead),
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ValidationRequests(
      reader.readObject(RollupValidationRequests),
      reader.readArray(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_KEY_VALIDATION_REQUESTS_PER_TX, ScopedKeyValidationRequestAndGenerator),
      reader.readArray(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return ValidationRequests.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new ValidationRequests(
      RollupValidationRequests.empty(),
      makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ScopedReadRequest.empty),
      makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest.empty),
      makeTuple(MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, ScopedReadRequest.empty),
      makeTuple(MAX_KEY_VALIDATION_REQUESTS_PER_TX, ScopedKeyValidationRequestAndGenerator.empty),
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty),
    );
  }

  [inspect.custom]() {
    return `ValidationRequests {
  forRollup: ${inspect(this.forRollup)},
  noteHashReadRequests: [${this.noteHashReadRequests
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  nullifierReadRequests: [${this.nullifierReadRequests
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  nullifierNonExistentReadRequests: [${this.nullifierNonExistentReadRequests
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  scopedKeyValidationRequestsAndGenerators: [${this.scopedKeyValidationRequestsAndGenerators
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  publicDataReads: [${this.publicDataReads
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}]
}`;
  }
}
