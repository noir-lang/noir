import { randomBytes } from '../crypto/index.js';

export class AztecAddress {
  public static SIZE_IN_BYTES = 32;
  public static ZERO = new AztecAddress(Buffer.alloc(AztecAddress.SIZE_IN_BYTES));

  constructor(private buffer: Buffer) {
    if (buffer.length !== AztecAddress.SIZE_IN_BYTES) {
      throw new Error(`Expect buffer size to be ${AztecAddress.SIZE_IN_BYTES}. Got ${buffer.length}.`);
    }
  }

  public static fromString(address: string) {
    return new AztecAddress(Buffer.from(address.replace(/^0x/i, ''), 'hex'));
  }

  public static random() {
    return new AztecAddress(randomBytes(64));
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
