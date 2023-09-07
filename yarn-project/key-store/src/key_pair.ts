import { GrumpkinPrivateKey, GrumpkinScalar } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { KeyPair, PublicKey } from '@aztec/types';

/**
 * The ConstantKeyPair class is an implementation of the KeyPair interface, which allows generation and management of
 * a constant public and private key pair. It provides methods for creating a random instance of the key pair,
 * retrieving the public key, getting the private key. This class ensures the persistence and consistency of
 * the generated keys, making it suitable for cryptographic operations where constant key pairs are required.
 */
export class ConstantKeyPair implements KeyPair {
  /**
   * Generate a random ConstantKeyPair instance using the .
   * The random private key is generated using 32 random bytes, and the corresponding public key is calculated
   * by multiplying the Grumpkin generator point with the private key. This function provides an efficient
   * way of generating unique key pairs for cryptographic purposes.
   *
   * @param curve - The curve used for elliptic curve cryptography operations.
   * @returns A randomly generated ConstantKeyPair instance.
   */
  public static random(curve: Grumpkin) {
    const privateKey = GrumpkinScalar.random();
    const publicKey = curve.mul(curve.generator(), privateKey);
    return new ConstantKeyPair(publicKey, privateKey);
  }

  /**
   * Creates a new instance from a private key.
   * @param curve - The curve used for elliptic curve cryptography operations.
   * @param signer - The signer to be used on the account.
   * @param privateKey - The private key.
   * @returns A new instance.
   */
  public static fromPrivateKey(curve: Grumpkin, privateKey: GrumpkinPrivateKey) {
    const publicKey = curve.mul(curve.generator(), privateKey);
    return new ConstantKeyPair(publicKey, privateKey);
  }

  constructor(private publicKey: PublicKey, private privateKey: GrumpkinPrivateKey) {}

  public getPublicKey(): PublicKey {
    return this.publicKey;
  }

  public getPrivateKey() {
    return Promise.resolve(this.privateKey);
  }
}
