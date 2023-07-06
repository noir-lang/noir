import { Point } from '@aztec/foundation/fields';

/** Represents a user public key. */
export type PublicKey = Point;

/**
 * Represents a secure storage for managing keys.
 * Provides functionality to create and retrieve accounts, private and public keys,
 */
export interface KeyStore {
  createAccount(): Promise<PublicKey>;
  addAccount(privKey: Buffer): PublicKey;
  getAccounts(): Promise<PublicKey[]>;
  getAccountPrivateKey(pubKey: PublicKey): Promise<Buffer>;
}
