import { randomBytes } from '../crypto/random/index.js';

export class TxHash {
  constructor(private buffer: Buffer) {
    if (buffer.length !== 32) {
      throw new Error('Invalid hash buffer.');
    }
  }

  static fromBuffer(buffer: Buffer) {
    return new TxHash(buffer);
  }

  static deserialize(buffer: Buffer, offset: number) {
    return { elem: new TxHash(buffer.slice(offset, offset + 32)), adv: 32 };
  }

  public static fromString(hash: string) {
    return new TxHash(Buffer.from(hash.replace(/^0x/i, ''), 'hex'));
  }

  public static random() {
    return new TxHash(randomBytes(32));
  }

  equals(rhs: TxHash) {
    return this.toBuffer().equals(rhs.toBuffer());
  }

  toBuffer() {
    return this.buffer;
  }

  toString() {
    return `0x${this.toBuffer().toString('hex')}`;
  }
}
