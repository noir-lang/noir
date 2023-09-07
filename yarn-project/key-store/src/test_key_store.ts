import { GrumpkinPrivateKey } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { KeyPair, KeyStore, PublicKey } from '@aztec/types';

import { ConstantKeyPair } from './key_pair.js';

/**
 * TestKeyStore is an implementation of the KeyStore interface, used for managing key pairs in a testing environment.
 * It should be utilized in testing scenarios where secure key management is not required, and ease-of-use is prioritized.
 */
export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];
  constructor(private curve: Grumpkin) {}

  public addAccount(privKey: GrumpkinPrivateKey): PublicKey {
    const keyPair = ConstantKeyPair.fromPrivateKey(this.curve, privKey);

    // check if private key has already been used
    const account = this.accounts.find(a => a.getPublicKey().equals(keyPair.getPublicKey()));
    if (account) {
      return account.getPublicKey();
    }

    this.accounts.push(keyPair);
    return keyPair.getPublicKey();
  }

  public createAccount(): Promise<PublicKey> {
    const keyPair = ConstantKeyPair.random(this.curve);
    this.accounts.push(keyPair);
    return Promise.resolve(keyPair.getPublicKey());
  }

  public getAccounts(): Promise<PublicKey[]> {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey()));
  }

  public getAccountPrivateKey(pubKey: PublicKey): Promise<GrumpkinPrivateKey> {
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
