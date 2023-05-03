import { BufferReader } from '@aztec/foundation/serialize';
import { serializeToBuffer } from '../utils/serialize.js';

export class CircuitError extends Error {
  constructor(public code: number, public message: string) {
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
