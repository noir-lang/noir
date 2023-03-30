import { randomBytes } from '@aztec/foundation';

export class Signature {
  public static SIZE = 64;

  public static random() {
    return new Signature(randomBytes(Signature.SIZE));
  }

  constructor(public readonly buffer: Buffer) {}
}
