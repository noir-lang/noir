import { PrivateKey, PublicKey } from '@aztec/circuits.js';

/**
 * Represents a cryptographic public-private key pair.
 * Provides functionality to generate, access, and sign messages using the key pair.
 */
export interface KeyPair {
  getPublicKey(): PublicKey;
  getPrivateKey(): Promise<PrivateKey>;
}
