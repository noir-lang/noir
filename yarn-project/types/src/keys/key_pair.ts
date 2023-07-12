import { Point } from '@aztec/circuits.js';

/**
 * Represents a cryptographic public-private key pair.
 * Provides functionality to generate, access, and sign messages using the key pair.
 */
export interface KeyPair {
  getPublicKey(): Point;
  getPrivateKey(): Promise<Buffer>;
}
