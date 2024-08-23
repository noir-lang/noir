import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { PreviousRollupBlockData } from './previous_rollup_block_data.js';

/**
 * Represents inputs of the block merge rollup circuit.
 */
export class BlockMergeRollupInputs {
  constructor(
    /**
     * Previous rollup data from the 2 block merge or block root rollup circuits that preceded this merge rollup circuit.
     */
    public previousRollupData: [PreviousRollupBlockData, PreviousRollupBlockData],
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
   * @returns A new BlockMergeRollupInputs instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new BlockMergeRollupInputs([
      reader.readObject(PreviousRollupBlockData),
      reader.readObject(PreviousRollupBlockData),
    ]);
  }

  /**
   * Deserializes the inputs from a hex string.
   * @param str - A hex string to deserialize from.
   * @returns A new BlockMergeRollupInputs instance.
   */
  static fromString(str: string) {
    return BlockMergeRollupInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}
