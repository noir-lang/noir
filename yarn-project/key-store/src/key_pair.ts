import { Curve } from '@aztec/circuits.js/barretenberg';
import { randomBytes } from '@aztec/foundation/crypto';
import { Point } from '@aztec/foundation/fields';
import { KeyPair, PublicKey } from '@aztec/types';

/**
 * The ConstantKeyPair class is an implementation of the KeyPair interface, which allows generation and management of a constant public and private key pair. It provides methods for creating a random instance of the key pair, retrieving the public key, getting the private key. This class ensures the persistence and consistency of the generated keys, making it suitable for cryptographic operations where constant key pairs are required.
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
  public static random(curve: Curve) {
    const privateKey = randomBytes(32);
    const publicKey = Point.fromBuffer(curve.mul(curve.generator(), privateKey));
    return new ConstantKeyPair(publicKey, privateKey);
  }

  /**
   * Creates a new instance from a private key.
   * @param curve - The curve used for elliptic curve cryptography operations.
   * @param signer - The signer to be used on the account.
   * @param privateKey - The private key.
   * @returns A new instance.
   */
  public static fromPrivateKey(curve: Curve, privateKey: Buffer) {
    const publicKey = Point.fromBuffer(curve.mul(curve.generator(), privateKey));
    return new ConstantKeyPair(publicKey, privateKey);
  }

  constructor(private publicKey: PublicKey, private privateKey: Buffer) {}

  /**
   * Retrieve the public key from the KeyPair instance.
   * The returned public key is a PublicKey object which represents a point on the elliptic curve secp256k1.
   *
   * @returns The public key as an elliptic curve point.
   */
  public getPublicKey() {
    return this.publicKey;
  }

  /**
   * Retrieves the private key of the KeyPair instance.
   * The function returns a Promise that resolves to a Buffer containing the private key.
   *
   * @returns A Promise that resolves to a Buffer containing the private key.
   */
  public getPrivateKey() {
    return Promise.resolve(this.privateKey);
  }
}
