import { BufferReader } from '@aztec/foundation';
import { serializeToBuffer } from '../utils/serialize.js';

export class CircuitError {
  constructor(public code: number, public message: string) {}

  toBuffer() {
    return serializeToBuffer(this.code, this.message);
  }

  static fromBuffer(buffer: Buffer | BufferReader): CircuitError {
    const reader = BufferReader.asReader(buffer);
    return new CircuitError(reader.readUInt16(), reader.readString());
  }
}
