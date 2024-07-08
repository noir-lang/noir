import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { type NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, NUM_MSGS_PER_BASE_PARITY } from '../../constants.gen.js';

export class BaseParityInputs {
  constructor(
    /** Aggregated proof of all the parity circuit iterations. */
    public readonly msgs: Tuple<Fr, typeof NUM_MSGS_PER_BASE_PARITY>,
    /** Root of the VK tree */
    public readonly vkTreeRoot: Fr,
  ) {}

  public static fromSlice(
    array: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
    index: number,
    vkTreeRoot: Fr,
  ): BaseParityInputs {
    const start = index * NUM_MSGS_PER_BASE_PARITY;
    const end = start + NUM_MSGS_PER_BASE_PARITY;
    const msgs = array.slice(start, end);
    return new BaseParityInputs(msgs as Tuple<Fr, typeof NUM_MSGS_PER_BASE_PARITY>, vkTreeRoot);
  }

  /** Serializes the inputs to a buffer. */
  toBuffer() {
    return serializeToBuffer(this.msgs, this.vkTreeRoot);
  }

  /** Serializes the inputs to a hex string. */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes the inputs from a buffer.
   * @param buffer - The buffer to deserialize from.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new BaseParityInputs(reader.readArray(NUM_MSGS_PER_BASE_PARITY, Fr), Fr.fromBuffer(reader));
  }

  /**
   * Deserializes the inputs from a hex string.
   * @param str - The hex string to deserialize from.
   * @returns - The deserialized inputs.
   */
  static fromString(str: string) {
    return BaseParityInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}
