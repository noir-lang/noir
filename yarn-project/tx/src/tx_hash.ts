export class TxHash {
  public static SIZE = 32;

  constructor(public readonly buffer: Buffer) {}

  public equals(rhs: TxHash) {
    return this.buffer.equals(rhs.buffer);
  }
}
