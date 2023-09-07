import { GrumpkinPrivateKey, PublicKey } from '@aztec/circuits.js';

/**
 * Represents a cryptographic public-private key pair.
 * Provides functionality to generate, access, and sign messages using the key pair.
 */
export interface KeyPair {
  /**
   * Retrieve the public key from the KeyPair instance.
   * The returned public key is a PublicKey object which represents a point on an elliptic curve.
   * @returns The public key as an elliptic curve point.
   */
  getPublicKey(): PublicKey;
  /**
   * Retrieves the private key of the KeyPair instance.
   * The function returns a Promise that resolves to a Buffer containing the private key.
   * @returns A Promise that resolves to a Buffer containing the private key.
   */
  getPrivateKey(): Promise<GrumpkinPrivateKey>;
}
