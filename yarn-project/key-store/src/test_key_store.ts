import { GrumpkinPrivateKey, GrumpkinScalar, Point } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { AztecKVStore, AztecMap } from '@aztec/kv-store';
import { KeyPair, KeyStore, PublicKey } from '@aztec/types';

import { ConstantKeyPair } from './key_pair.js';

/**
 * TestKeyStore is an implementation of the KeyStore interface, used for managing key pairs in a testing environment.
 * It should be utilized in testing scenarios where secure key management is not required, and ease-of-use is prioritized.
 */
export class TestKeyStore implements KeyStore {
  #keys: AztecMap<string, Buffer>;

  constructor(private curve: Grumpkin, database: AztecKVStore) {
    this.#keys = database.createMap('key_store');
  }

  public async addAccount(privKey: GrumpkinPrivateKey): Promise<PublicKey> {
    const keyPair = ConstantKeyPair.fromPrivateKey(this.curve, privKey);
    await this.#keys.setIfNotExists(keyPair.getPublicKey().toString(), keyPair.getPrivateKey().toBuffer());
    return keyPair.getPublicKey();
  }

  public async createAccount(): Promise<PublicKey> {
    const keyPair = ConstantKeyPair.random(this.curve);
    await this.#keys.set(keyPair.getPublicKey().toString(), keyPair.getPrivateKey().toBuffer());
    return keyPair.getPublicKey();
  }

  public getAccounts(): Promise<PublicKey[]> {
    const range = Array.from(this.#keys.keys());
    return Promise.resolve(range.map(key => Point.fromString(key)));
  }

  public getAccountPrivateKey(pubKey: PublicKey): Promise<GrumpkinPrivateKey> {
    const account = this.getAccount(pubKey);
    return Promise.resolve(account.getPrivateKey());
  }

  /**
   * Retrieve the KeyPair object associated with a given pub key.
   * Searches through the 'accounts' array for a matching public key and returns the corresponding account (KeyPair).
   * Throws an error if no matching account is found in the 'accounts'.
   *
   * @param pubKey - The public key of the account to retrieve.
   * @returns The KeyPair object associated with the provided key.
   */
  private getAccount(pubKey: PublicKey): KeyPair {
    const privKey = this.#keys.get(pubKey.toString());
    if (!privKey) {
      throw new Error(
        'Unknown account.\nSee docs for context: https://docs.aztec.network/dev_docs/contracts/common_errors#unknown-contract-error',
      );
    }
    return ConstantKeyPair.fromPrivateKey(this.curve, GrumpkinScalar.fromBuffer(privKey));
  }
}
