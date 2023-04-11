import { toBigInt } from '@aztec/foundation';

export class TxHash {
  public static SIZE = 32;

  constructor(public readonly buffer: Buffer) {}

  public equals(rhs: TxHash) {
    return this.buffer.equals(rhs.buffer);
  }

  /**
   * Convert this hash to a hex string.
   * @returns The hex string.
   */
  public toString() {
    return this.buffer.toString('hex');
  }
  /**
   * Convert this hash to a big int.
   * @returns The big int.
   */
  public toBigInt() {
    return toBigInt(this.buffer);
  }
}
