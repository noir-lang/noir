import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { Fr } from '../fields/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';

export class AztecAddress {
  static SIZE_IN_BYTES = 32;
  static ZERO = new AztecAddress(Buffer.alloc(AztecAddress.SIZE_IN_BYTES));
  static MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001n;
  static MAX_VALUE = AztecAddress.MODULUS - 1n;

  constructor(public readonly buffer: Buffer) {
    const value = toBigIntBE(buffer);
    if (value > AztecAddress.MAX_VALUE) {
      throw new Error(`AztecAddress out of range ${value}.`);
    }
  }

  static random() {
    return new AztecAddress(toBufferBE(Fr.random().value, AztecAddress.SIZE_IN_BYTES));
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new this(reader.readBytes(this.SIZE_IN_BYTES));
  }

  static fromString(address: string) {
    return new AztecAddress(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
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

  equals(rhs: AztecAddress) {
    return this.buffer.equals(rhs.buffer);
  }
}
