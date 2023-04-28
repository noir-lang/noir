import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { BufferReader, Fr } from '@aztec/foundation';

/**
 * Write operations on the public state tree.
 */
export class PublicDataWrite {
  constructor(public readonly leafIndex: Fr, public readonly newValue: Fr) {}

  static from(args: { leafIndex: Fr; newValue: Fr }) {
    return new PublicDataWrite(args.leafIndex, args.newValue);
  }

  toBuffer() {
    return serializeToBuffer(this.leafIndex, this.newValue);
  }

  isEmpty() {
    return this.leafIndex.isZero() && this.newValue.isZero();
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataWrite(reader.readFr(), reader.readFr());
  }

  static empty() {
    return new PublicDataWrite(Fr.ZERO, Fr.ZERO);
  }

  static random() {
    return new PublicDataWrite(Fr.random(), Fr.random());
  }
}
