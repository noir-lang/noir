import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { Fr } from './index.js';
import { BufferReader } from '../serialize/buffer_reader.js';
import { AztecAddress } from '../aztec-address/index.js';

export class Point {
  static SIZE_IN_BYTES = 64;
  static ZERO = new Point(Buffer.alloc(Point.SIZE_IN_BYTES));
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;
  static MAX_VALUE = Point.MODULUS - 1n;

  constructor(public readonly buffer: Buffer) {
    const coordinateX = toBigIntBE(buffer.subarray(0, 32));
    const coordinateY = toBigIntBE(buffer.subarray(32, 64));
    if (coordinateX > Point.MAX_VALUE) {
      throw new Error(`Coordinate x out of range: ${coordinateX}.`);
    }
    if (coordinateY > Point.MAX_VALUE) {
      throw new Error(`Coordinate y out of range: ${coordinateY}.`);
    }
  }

  static random() {
    return new Point(toBufferBE(Fr.random().value, Point.SIZE_IN_BYTES));
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }

  static fromString(address: string) {
    return new Point(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  toBuffer() {
    return this.buffer;
  }

  toString() {
    return '0x' + this.buffer.toString('hex');
  }

  toShortString() {
    const str = this.toString();
    return `${str.slice(0, 10)}...${str.slice(-4)}`;
  }

  equals(rhs: Point) {
    return this.buffer.equals(rhs.buffer);
  }

  toAddress(): AztecAddress {
    return AztecAddress.fromBuffer(this.buffer.slice(0, AztecAddress.SIZE_IN_BYTES));
  }
}
