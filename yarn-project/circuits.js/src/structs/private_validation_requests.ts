import { makeTuple } from '@aztec/foundation/array';
import { arraySerializedSizeOfNonEmpty } from '@aztec/foundation/collection';
import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

import {
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
} from '../constants.gen.js';
import { OptionalNumber } from './optional_number.js';
import { ScopedReadRequest } from './read_request.js';
import { RollupValidationRequests } from './rollup_validation_requests.js';
import { ScopedKeyValidationRequestAndGenerator } from './scoped_key_validation_request_and_generator.js';

/**
 * Validation requests accumulated during the execution of the transaction.
 */
export class PrivateValidationRequests {
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
     * All the key validation requests made in this transaction.
     */
    public scopedKeyValidationRequestsAndGenerators: Tuple<
      ScopedKeyValidationRequestAndGenerator,
      typeof MAX_KEY_VALIDATION_REQUESTS_PER_TX
    >,
    /**
     * The counter to split the data for squashing.
     * A revertible nullifier and a non-revertible note hash will not be squashed.
     * It should be the "final" minRevertibleSideEffectCounter of a tx.
     */
    public splitCounter: OptionalNumber,
  ) {}

  getSize() {
    return (
      this.forRollup.getSize() +
      arraySerializedSizeOfNonEmpty(this.noteHashReadRequests) +
      arraySerializedSizeOfNonEmpty(this.nullifierReadRequests) +
      arraySerializedSizeOfNonEmpty(this.scopedKeyValidationRequestsAndGenerators) +
      this.splitCounter.getSize()
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.forRollup,
      this.noteHashReadRequests,
      this.nullifierReadRequests,
      this.scopedKeyValidationRequestsAndGenerators,
      this.splitCounter,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new PrivateValidationRequests(
      reader.readObject(RollupValidationRequests),
      reader.readArray(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_KEY_VALIDATION_REQUESTS_PER_TX, ScopedKeyValidationRequestAndGenerator),
      reader.readObject(OptionalNumber),
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateValidationRequests(
      reader.readObject(RollupValidationRequests),
      reader.readArray(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest),
      reader.readArray(MAX_KEY_VALIDATION_REQUESTS_PER_TX, ScopedKeyValidationRequestAndGenerator),
      reader.readObject(OptionalNumber),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return PrivateValidationRequests.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new PrivateValidationRequests(
      RollupValidationRequests.empty(),
      makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ScopedReadRequest.empty),
      makeTuple(MAX_NULLIFIER_READ_REQUESTS_PER_TX, ScopedReadRequest.empty),
      makeTuple(MAX_KEY_VALIDATION_REQUESTS_PER_TX, ScopedKeyValidationRequestAndGenerator.empty),
      OptionalNumber.empty(),
    );
  }

  [inspect.custom]() {
    return `PrivateValidationRequests {
  forRollup: ${inspect(this.forRollup)},
  noteHashReadRequests: [${this.noteHashReadRequests
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  nullifierReadRequests: [${this.nullifierReadRequests
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  scopedKeyValidationRequestsAndGenerators: [${this.scopedKeyValidationRequestsAndGenerators
    .filter(x => !x.isEmpty())
    .map(h => inspect(h))
    .join(', ')}],
  splitCounter: ${this.splitCounter}
  `;
  }
}
