import { AztecAddress, Fr } from '@aztec/circuits.js';
import { Signature } from '../circuits.js';
import { randomBytes } from '../foundation.js';

export interface KeyPair {
  getPublicKey(): AztecAddress;
  getPrivateKey(): Promise<Buffer>;
  signMessage(message: Buffer): Promise<Signature>;
}

export class ConstantKeyPair implements KeyPair {
  public static random() {
    const privateKey = randomBytes(32);
    const publicKey = AztecAddress.random();
    return new ConstantKeyPair(publicKey, privateKey);
  }

  constructor(private publicKey: AztecAddress, private privateKey: Buffer) {}

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
