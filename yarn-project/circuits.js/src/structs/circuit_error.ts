import { BufferReader } from '@aztec/foundation/serialize';

import { serializeToBuffer } from '../utils/serialize.js';

/**
 * A class representing a circuit error.
 *
 * Used to make C++ errors more readable on the TS side.
 */
export class CircuitError extends Error {
  constructor(
    /**
     * Code identifying the error.
     */
    public code: number,
    /**
     * Message describing the error.
     */
    public message: string,
  ) {
    super(message);
  }

  toBuffer() {
    return serializeToBuffer(this.code, this.message);
  }

  static fromBuffer(buffer: Buffer | BufferReader): CircuitError {
    const reader = BufferReader.asReader(buffer);
    return new CircuitError(reader.readUInt16(), reader.readString());
  }
}
