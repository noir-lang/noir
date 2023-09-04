import { Fr } from './index.js';
import { BufferReader } from '../serialize/buffer_reader.js';

export class Point {
  static SIZE_IN_BYTES = 64;
  static EMPTY = new Point(Fr.ZERO, Fr.ZERO);

  constructor(public readonly x: Fr, public readonly y: Fr) {}

  static random() {
    // TODO: This is not a point on the curve!
    return new Point(Fr.random(), Fr.random());
  }

  static fromBuffer(buffer: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  static fromString(address: string) {
    return Point.fromBuffer(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  toBuffer() {
    return Buffer.concat([this.x.toBuffer(), this.y.toBuffer()]);
  }

  toString() {
    return '0x' + this.toBuffer().toString('hex');
  }

  equals(rhs: Point) {
    return this.x.equals(rhs.x) && this.y.equals(rhs.y);
  }
}
