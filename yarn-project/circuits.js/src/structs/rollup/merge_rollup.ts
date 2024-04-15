import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { PreviousRollupData } from './previous_rollup_data.js';

/**
 * Represents inputs of the merge rollup circuit.
 */
export class MergeRollupInputs {
  constructor(
    /**
     * Previous rollup data from the 2 merge or base rollup circuits that preceded this merge rollup circuit.
     */
    public previousRollupData: [PreviousRollupData, PreviousRollupData],
  ) {}

  /**
   * Serializes the inputs to a buffer.
   * @returns The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.previousRollupData);
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The instance serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes the inputs from a buffer.
   * @param buffer - The buffer to deserialize from.
   * @returns A new MergeRollupInputs instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new MergeRollupInputs([reader.readObject(PreviousRollupData), reader.readObject(PreviousRollupData)]);
  }

  /**
   * Deserializes the inputs from a hex string.
   * @param str - A hex string to deserialize from.
   * @returns A new MergeRollupInputs instance.
   */
  static fromString(str: string) {
    return MergeRollupInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}
