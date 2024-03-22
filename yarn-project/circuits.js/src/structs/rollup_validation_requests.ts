import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

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

  toBuffer() {
    return serializeToBuffer(this.maxBlockNumber);
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
