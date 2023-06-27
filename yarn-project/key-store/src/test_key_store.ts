import { Curve, Signature, Signer } from '@aztec/circuits.js/barretenberg';
import { ConstantKeyPair, KeyPair } from './key_pair.js';
import { KeyStore, PublicKey } from './key_store.js';
import { Point } from '@aztec/circuits.js';

/**
 * TestKeyStore is an implementation of the KeyStore interface, used for managing key pairs in a testing environment.
 * It should be utilized in testing scenarios where secure key management is not required, and ease-of-use is prioritized.
 */
export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];

  constructor() {}

  /**
   * Adds an account to the key store from the provided private key.
   * @param curve - The curve to use for generating the public key.
   * @param signer - The signer to use on the account.
   * @param privKey - The private key of the account.
   * @returns - The account's public key.
   */
  public addAccount(curve: Curve, signer: Signer, privKey: Buffer): PublicKey {
    const keyPair = ConstantKeyPair.fromPrivateKey(curve, signer, privKey);
    this.accounts.push(keyPair);
    return keyPair.getPublicKey();
  }

  /**
   * Adds a new account to the TestKeyStore with a randomly generated ConstantKeyPair.
   * The account will have its own private and public key pair, which can be used for signing transactions.
   * @param curve - The curve to use for generating the public key.
   * @param signer - The signer to use on the account.
   * @returns A promise that resolves to the newly created account's AztecAddress.
   */
  public createAccount(curve: Curve, signer: Signer): Promise<Point> {
    const keyPair = ConstantKeyPair.random(curve, signer);
    this.accounts.push(keyPair);
    return Promise.resolve(keyPair.getPublicKey());
  }

  /**
   * Retrieves the public addresses of all accounts stored in the TestKeyStore.
   * The returned addresses are instances of `AztecAddress` and can be used for subsequent operations
   * such as signing transactions or fetching public/private keys.
   *
   * @returns A Promise that resolves to an array of AztecAddress instances.
   */
  public getAccounts() {
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
   * Sign a buffer using the private key of the sender account.
   * The 'signMessage' method of the account private key is called internally to generate the signature.
   * Throws an error if the sender account is not found in the TestKeyStore.
   *
   * @param what - What to sign.
   * @param from - What key to use.
   * @returns The signed message.
   */
  public async sign(what: Buffer, from: PublicKey): Promise<Signature> {
    const account = this.getAccount(from);
    return await account.sign(what);
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
