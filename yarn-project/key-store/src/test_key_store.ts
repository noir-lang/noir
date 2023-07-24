import { Curve } from '@aztec/circuits.js/barretenberg';
import { KeyPair, KeyStore, PublicKey } from '@aztec/types';

import { ConstantKeyPair } from './key_pair.js';

/**
 * TestKeyStore is an implementation of the KeyStore interface, used for managing key pairs in a testing environment.
 * It should be utilized in testing scenarios where secure key management is not required, and ease-of-use is prioritized.
 */
export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];
  constructor(private curve: Curve) {}

  /**
   * Adds an account to the key store from the provided private key.
   * @param curve - The curve to use for generating the public key.
   * @param privKey - The private key of the account.
   * @returns - The account's public key.
   */
  public addAccount(privKey: Buffer): PublicKey {
    const keyPair = ConstantKeyPair.fromPrivateKey(this.curve, privKey);

    // check if private key has already been used
    const account = this.accounts.find(a => a.getPublicKey().equals(keyPair.getPublicKey()));
    if (account) {
      return account.getPublicKey();
    }

    this.accounts.push(keyPair);
    return keyPair.getPublicKey();
  }

  /**
   * Adds a new account to the TestKeyStore with a randomly generated ConstantKeyPair.
   * The account will have its own private and public key pair, which can be used for signing transactions.
   * @param curve - The curve to use for generating the public key.
   * @returns A promise that resolves to the newly created account's AztecAddress.
   */
  public createAccount(): Promise<PublicKey> {
    const keyPair = ConstantKeyPair.random(this.curve);
    this.accounts.push(keyPair);
    return Promise.resolve(keyPair.getPublicKey());
  }

  /**
   * Retrieves the public keys of all accounts stored in the TestKeyStore.
   * The returned addresses are instances of `PublicKey` and can be used for subsequent operations
   * such as signing transactions or fetching public/private keys.
   *
   * @returns A Promise that resolves to an array of public keys instances.
   */
  public getAccounts(): Promise<PublicKey[]> {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey()));
  }

  /**
   * Retrieves the private key of the account associated with the specified AztecAddress.
   * Throws an error if the provided address is not found in the list of registered accounts.
   *
   * @param pubKey - The AztecAddress instance representing the account for which the private key is requested.
   * @returns A Promise that resolves to a Buffer containing the private key.
   * @deprecated We should not require a keystore to expose private keys in plain.
   */
  public getAccountPrivateKey(pubKey: PublicKey): Promise<Buffer> {
    const account = this.getAccount(pubKey);
    return account.getPrivateKey();
  }

  /**
   * Retrieve the KeyPair object associated with a given pub key.
   * Searches through the 'accounts' array for a matching public key and returns the corresponding account (KeyPair).
   * Throws an error if no matching account is found in the 'accounts'.
   *
   * @param pubKey - The public key of the account to retrieve.
   * @returns The KeyPair object associated with the provided key.
   */
  private getAccount(pubKey: PublicKey) {
    const account = this.accounts.find(a => a.getPublicKey().equals(pubKey));
    if (!account) {
      throw new Error('Unknown account.');
    }
    return account;
  }
}
