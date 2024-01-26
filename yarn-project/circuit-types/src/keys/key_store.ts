import { AztecAddress, GrumpkinPrivateKey, PublicKey } from '@aztec/circuits.js';

/**
 * Represents a secure storage for managing keys.
 * Provides functionality to create and retrieve accounts, private and public keys,
 */
export interface KeyStore {
  /**
   * Adds a new account with a randomly generated key pair.
   * The account will have its own private and public key pair, which can be used for signing transactions.
   * @returns A promise that resolves to the newly created account's AztecAddress.
   */
  createAccount(): Promise<PublicKey>;

  /**
   * Adds an account to the key store from the provided private key.
   * @param curve - The curve to use for generating the public key.
   * @param privKey - The private key of the account.
   * @returns - The account's public key.
   */
  addAccount(privKey: GrumpkinPrivateKey): Promise<PublicKey>;

  /**
   * Retrieves the public keys of all accounts stored.
   * The returned addresses are instances of `PublicKey` and can be used for subsequent operations
   * such as signing transactions or fetching public/private keys.
   * @returns A Promise that resolves to an array of public keys instances.
   */
  getAccounts(): Promise<PublicKey[]>;

  /**
   * Retrieves the private key of the account associated with the specified AztecAddress.
   * Throws an error if the provided address is not found in the list of registered accounts.
   * @param pubKey - The AztecAddress instance representing the account for which the private key is requested.
   * @returns A Promise that resolves to a Buffer containing the private key.
   * @deprecated We should not require a keystore to expose private keys in plain.
   */
  getAccountPrivateKey(pubKey: PublicKey): Promise<GrumpkinPrivateKey>;

  /**
   * Retrieves the nullifier secret key of the account associated with the specified AztecAddress.
   * Throws an error if the provided public key is not found in the list of registered accounts.
   * @param pubKey - The public key of the account for which the secret key is requested.
   * @returns A Promise that resolves to the nullifier secret key.
   */
  getNullifierSecretKey(pubKey: PublicKey): Promise<GrumpkinPrivateKey>;

  /**
   * Retrieves the nullifier secret key of the specified nullifier public key.
   * Throws an error if the provided public key is not associated with any of the registered accounts.
   *
   * @param nullifierPubKey - The nullifier public key.
   * @returns A Promise that resolves to the nullifier secret key.
   */
  getNullifierSecretKeyFromPublicKey(nullifierPubKey: PublicKey): Promise<GrumpkinPrivateKey>;

  /**
   * Retrieves the nullifier public key of the account associated with the specified AztecAddress.
   * Throws an error if the provided public key is not found in the list of registered accounts.
   * @param pubKey - The public key of the account for which the nullifier public key is requested.
   * @returns A Promise that resolves to the nullifier public key.
   */
  getNullifierPublicKey(pubKey: PublicKey): Promise<PublicKey>;

  /**
   * Retrieves the nullifier secret key for use in a specific contract.
   * Throws an error if the provided public key is not found in the list of registered accounts.
   * @param pubKey - The public key of the account for which the private key is requested.
   * @param contractAddress - The address of the contract requesting the nullifier key.
   * @returns A Promise that resolves to the nullifier secret key.
   */
  getSiloedNullifierSecretKey(pubKey: PublicKey, contractAddress: AztecAddress): Promise<GrumpkinPrivateKey>;
}
