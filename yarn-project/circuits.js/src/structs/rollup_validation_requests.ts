import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { MaxBlockNumber } from './max_block_number.js';

/**
 * Validation requests directed at the rollup, accumulated during the execution of the transaction.
 */
export class RollupValidationRequests {
  constructor(
    /**
     * The largest block number in which this transaction can be included.
     */
    public maxBlockNumber: MaxBlockNumber,
  ) {}

  getSize() {
    return this.toBuffer().length;
  }

  toBuffer() {
    return serializeToBuffer(this.maxBlockNumber);
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new RollupValidationRequests(MaxBlockNumber.fromFields(reader));
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new RollupValidationRequests(reader.readObject(MaxBlockNumber));
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return RollupValidationRequests.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new RollupValidationRequests(MaxBlockNumber.empty());
  }
}
