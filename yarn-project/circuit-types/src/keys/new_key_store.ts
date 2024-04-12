import { type AztecAddress, type Fr, type PartialAddress, type PublicKey } from '@aztec/circuits.js';

/**
 * Represents a secure storage for managing keys.
 */
export interface NewKeyStore {
  /**
   * Creates a new account from a randomly generated secret key.
   * @returns A promise that resolves to the newly created account's AztecAddress.
   */
  createAccount(): Promise<AztecAddress>;

  /**
   * Adds an account to the key store from the provided secret key.
   * @param sk - The secret key of the account.
   * @param partialAddress - The partial address of the account.
   * @returns The account's address.
   */
  addAccount(sk: Fr, partialAddress: PartialAddress): Promise<AztecAddress>;

  /**
   * Gets the master nullifier public key for a given account.
   * @throws If the account does not exist in the key store.
   * @param account - The account address for which to retrieve the master nullifier public key.
   * @returns The master nullifier public key for the account.
   */
  getMasterNullifierPublicKey(account: AztecAddress): Promise<PublicKey>;

  /**
   * Gets the master incoming viewing public key for a given account.
   * @throws If the account does not exist in the key store.
   * @param account - The account address for which to retrieve the master incoming viewing public key.
   * @returns The master incoming viewing public key for the account.
   */
  getMasterIncomingViewingPublicKey(account: AztecAddress): Promise<PublicKey>;

  /**
   * Retrieves the master outgoing viewing key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the master outgoing viewing key for.
   * @returns A Promise that resolves to the master outgoing viewing key.
   */
  getMasterOutgoingViewingPublicKey(account: AztecAddress): Promise<PublicKey>;

  /**
   * Retrieves the master tagging key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the master tagging key for.
   * @returns A Promise that resolves to the master tagging key.
   */
  getMasterTaggingPublicKey(account: AztecAddress): Promise<PublicKey>;
}
