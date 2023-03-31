import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { Point } from '@aztec/foundation';
import { Signature } from '../circuits.js';
import { randomBytes } from '../foundation.js';

export interface KeyPair {
  getPublicKey(): Point;
  getPrivateKey(): Promise<Buffer>;
  signMessage(message: Buffer): Promise<Signature>;
}

export class ConstantKeyPair implements KeyPair {
  public static random(grumpkin: Grumpkin) {
    const privateKey = randomBytes(32);
    const publicKey = Point.fromBuffer(grumpkin.mul(Grumpkin.generator, privateKey));
    return new ConstantKeyPair(publicKey, privateKey);
  }

  constructor(private publicKey: Point, private privateKey: Buffer) {}

  public getPublicKey() {
    return this.publicKey;
  }

  public getPrivateKey() {
    return Promise.resolve(this.privateKey);
  }

  public signMessage(message: Buffer) {
    if (!message.length) {
      throw new Error('Cannot sign over empty message.');
    }

    return Promise.resolve(Signature.random());
  }
}
