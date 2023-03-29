import { randomBytes } from '../crypto/index.js';
import { BufferReader } from '../index.js';

export class AztecAddress {
  public static SIZE_IN_BYTES = 32;
  public static ZERO = new AztecAddress(Buffer.alloc(AztecAddress.SIZE_IN_BYTES));

  constructor(private buffer: Buffer) {
    if (buffer.length !== AztecAddress.SIZE_IN_BYTES) {
      throw new Error(`Expect buffer size to be ${AztecAddress.SIZE_IN_BYTES}. Got ${buffer.length}.`);
    }
  }

  static fromBuffer(bufferOrReader: Buffer | BufferReader) {
    const reader = BufferReader.asReader(bufferOrReader);
    return new AztecAddress(reader.readBytes(AztecAddress.SIZE_IN_BYTES));
  }

  public static fromString(address: string) {
    return new AztecAddress(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  public static random() {
    return new AztecAddress(randomBytes(AztecAddress.SIZE_IN_BYTES));
  }

  public equals(rhs: AztecAddress) {
    return this.buffer.equals(rhs.toBuffer());
  }

  public toBuffer() {
    return this.buffer;
  }

  public toString() {
    return `0x${this.buffer.toString('hex')}`;
  }

  public toShortString() {
    const str = this.toString();
    return `${str.slice(0, 10)}...${str.slice(-4)}`;
  }
}
