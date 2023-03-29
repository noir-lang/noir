import { toBigIntBE, toBufferBE } from '../bigint-buffer/index.js';
import { Fr } from '../fields/index.js';
import { BufferReader } from '../serialize/buffer_reader.js';

export class AztecAddress {
  static SIZE_IN_BYTES = 32;
  static ZERO = new AztecAddress(Buffer.alloc(AztecAddress.SIZE_IN_BYTES));

  constructor(public readonly buffer: Buffer) {
    const value = toBigIntBE(buffer);
    if (value > Fr.MAX_VALUE) {
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
