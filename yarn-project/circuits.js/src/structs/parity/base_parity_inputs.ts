import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { type NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, NUM_MSGS_PER_BASE_PARITY } from '../../constants.gen.js';

export class BaseParityInputs {
  constructor(
    /** Aggregated proof of all the parity circuit iterations. */
    public readonly msgs: Tuple<Fr, typeof NUM_MSGS_PER_BASE_PARITY>,
  ) {}

  public static fromSlice(
    array: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
    index: number,
  ): BaseParityInputs {
    const start = index * NUM_MSGS_PER_BASE_PARITY;
    const end = start + NUM_MSGS_PER_BASE_PARITY;
    const msgs = array.slice(start, end);
    return new BaseParityInputs(msgs as Tuple<Fr, typeof NUM_MSGS_PER_BASE_PARITY>);
  }

  toBuffer() {
    return serializeToBuffer(this.msgs);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new BaseParityInputs(reader.readArray(NUM_MSGS_PER_BASE_PARITY, Fr));
  }
}
