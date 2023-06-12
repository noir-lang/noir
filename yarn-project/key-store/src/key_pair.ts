import { EcdsaSignature } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { randomBytes } from '@aztec/foundation/crypto';
import { Point } from '@aztec/foundation/fields';
import { secp256k1 } from '@noble/curves/secp256k1';

/**
 * Represents a cryptographic public-private key pair.
 * Provides functionality to generate, access, and sign messages using the key pair.
 */
export interface KeyPair {
  getPublicKey(): Point;
  getPrivateKey(): Promise<Buffer>;
  ecdsaSign(message: Buffer): Promise<EcdsaSignature>;
}

/**
 * The ConstantKeyPair class is an implementation of the KeyPair interface, which allows generation and management of a constant public and private key pair. It provides methods for creating a random instance of the key pair, retrieving the public key, getting the private key, and signing messages securely using the ECDSA signature algorithm. This class ensures the persistence and consistency of the generated keys, making it suitable for cryptographic operations where constant key pairs are required.
 */
export class ConstantKeyPair implements KeyPair {
  /**
   * Generate a random ConstantKeyPair instance on a Grumpkin curve.
   * The random private key is generated using 32 random bytes, and the corresponding public key is calculated
   * by multiplying the Grumpkin generator point with the private key. This function provides an efficient
   * way of generating unique key pairs for cryptographic purposes.
   *
   * @param grumpkin - The Grumpkin curve used for elliptic curve cryptography.
   * @returns A randomly generated ConstantKeyPair instance.
   */
  public static random(grumpkin: Grumpkin) {
    const privateKey = randomBytes(32);
    const publicKey = Point.fromBuffer(grumpkin.mul(Grumpkin.generator, privateKey));
    return new ConstantKeyPair(publicKey, privateKey);
  }

  /**
   * Creates a new instance from a private key.
   * @param privateKey - The private key.
   * @returns A new instance.
   */
  public static async fromPrivateKey(privateKey: Buffer) {
    const grumpkin = await Grumpkin.new();
    const publicKey = Point.fromBuffer(grumpkin.mul(Grumpkin.generator, privateKey));
    return new ConstantKeyPair(publicKey, privateKey);
  }

  constructor(private publicKey: Point, private privateKey: Buffer) {}

  /**
   * Retrieve the public key from the KeyPair instance.
   * The returned public key is a Point object which represents a point on the elliptic curve secp256k1.
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

  /**
   * Sign a given message using the private key of the key pair.
   * The input 'message' should be a non-empty Buffer containing the data to be signed.
   * Throws an error if the input message is empty.
   *
   * @param message - The Buffer containing the data to be signed.
   * @returns A Promise that resolves to an EcdsaSignature instance representing the signature.
   */
  public async ecdsaSign(message: Buffer) {
    if (!message.length) {
      throw new Error('Cannot sign over empty message.');
    }
    const signature = secp256k1.sign(message, await this.getPrivateKey());
    if (signature.recovery === undefined) throw new Error(`Missing recovery from signature`);
    return EcdsaSignature.fromBigInts(signature.r, signature.s, signature.recovery);
  }
}
