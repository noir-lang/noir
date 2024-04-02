import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { NUM_BASE_PARITY_PER_ROOT_PARITY } from '../../constants.gen.js';
import { RootParityInput } from './root_parity_input.js';

export class RootParityInputs {
  constructor(
    /** Public inputs of children and their proofs. */
    public readonly children: Tuple<RootParityInput, typeof NUM_BASE_PARITY_PER_ROOT_PARITY>,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.children);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new RootParityInputs(reader.readArray(NUM_BASE_PARITY_PER_ROOT_PARITY, RootParityInput));
  }
}
