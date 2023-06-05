import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { AztecAddress } from '@aztec/circuits.js';
import { Point } from '@aztec/foundation/fields';
import { TxExecutionRequest } from '@aztec/types';
import { ConstantKeyPair, KeyPair } from './key_pair.js';
import { KeyStore } from './key_store.js';

/**
 * TestKeyStore is an implementation of the KeyStore interface, used for managing key pairs in a testing environment.
 * It should be utilized in testing scenarios where secure key management is not required, and ease-of-use is prioritized.
 */
export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];

  constructor(private grumpkin: Grumpkin) {}

  /**
   * Adds a new account to the TestKeyStore with a randomly generated ConstantKeyPair.
   * The account will have its own private and public key pair, which can be used for signing transactions.
   *
   * @returns A promise that resolves to the newly created account's AztecAddress.
   */
  public addAccount() {
    const keyPair = ConstantKeyPair.random(this.grumpkin);
    this.accounts.push(keyPair);
    return Promise.resolve(keyPair.getPublicKey().toAddress());
  }

  /**
   * Retrieves the public addresses of all accounts stored in the TestKeyStore.
   * The returned addresses are instances of `AztecAddress` and can be used for subsequent operations
   * such as signing transactions or fetching public/private keys.
   *
   * @returns A Promise that resolves to an array of AztecAddress instances.
   */
  public getAccounts() {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey().toAddress()));
  }

  /**
   * Retrieves the private key of the account associated with the specified AztecAddress.
   * Throws an error if the provided address is not found in the list of registered accounts.
   *
   * @param address - The AztecAddress instance representing the account for which the private key is requested.
   * @returns A Promise that resolves to a Buffer containing the private key.
   */
  public getAccountPrivateKey(address: AztecAddress): Promise<Buffer> {
    const account = this.getAccount(address);
    return account.getPrivateKey();
  }

  /**
   * Retrieve the public key of an account with a given address.
   * Searches for the corresponding account in the accounts array, and returns its public key.
   * If the account is not found, an error is thrown.
   *
   * @param address - The AztecAddress of the account whose public key is to be retrieved.
   * @returns A Promise that resolves with the Point instance representing the public key of the account.
   */
  public getAccountPublicKey(address: AztecAddress): Promise<Point> {
    const account = this.getAccount(address);
    return Promise.resolve(account.getPublicKey());
  }

  /**
   * Retrieves an array of public keys for all accounts stored in the TestKeyStore.
   * These public keys can be used for verifying signatures on transactions and messages.
   *
   * @returns A promise that resolves to an array of public keys associated with the accounts in the TestKeyStore.
   */
  public getSigningPublicKeys() {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey()));
  }

  /**
   * Sign a transaction request using the private key of the sender account.
   * The 'signMessage' method of the account private key is called internally to generate the signature.
   * Throws an error if the sender account is not found in the TestKeyStore.
   *
   * @param txRequest - The transaction request to be signed. It includes the sender, receiver, and other details.
   * @returns A Promise which resolves to the generated signature as a Buffer.
   */
  public signTxRequest(txRequest: TxExecutionRequest) {
    const account = this.getAccount(txRequest.from);
    // TODO - Define signing data.
    const signingData = txRequest.toBuffer();
    return account.signMessage(signingData);
  }

  /**
   * Retrieve the KeyPair object associated with a given address.
   * Searches through the 'accounts' array for a matching public key and returns the corresponding account (KeyPair).
   * Throws an error if no matching account is found in the 'accounts'.
   *
   * @param address - The address of the account to retrieve.
   * @returns The KeyPair object associated with the provided address.
   */
  private getAccount(address: AztecAddress) {
    const account = this.accounts.find(a => a.getPublicKey().toAddress().equals(address));
    if (!account) {
      throw new Error('Unknown account.');
    }
    return account;
  }
}
