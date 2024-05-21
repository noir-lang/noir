import {
  type AztecAddress,
  type CompleteAddress,
  type Fq,
  type Fr,
  type GrumpkinPrivateKey,
  type KeyGenerator,
  type KeyValidationRequest,
  type PartialAddress,
  type PublicKey,
} from '@aztec/circuits.js';

/**
 * Represents a secure storage for managing keys.
 */
export interface KeyStore {
  /**
   * Creates a new account from a randomly generated secret key.
   * @returns A promise that resolves to the newly created account's CompleteAddress.
   */
  createAccount(): Promise<CompleteAddress>;

  /**
   * Adds an account to the key store from the provided secret key.
   * @param sk - The secret key of the account.
   * @param partialAddress - The partial address of the account.
   * @returns The account's complete address.
   */
  addAccount(sk: Fr, partialAddress: PartialAddress): Promise<CompleteAddress>;

  /**
   * Retrieves addresses of accounts stored in the key store.
   * @returns A Promise that resolves to an array of account addresses.
   */
  getAccounts(): Promise<AztecAddress[]>;

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

  /**
   * Retrieves application incoming viewing secret key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the application incoming viewing secret key for.
   * @param app - The application address to retrieve the incoming viewing secret key for.
   * @returns A Promise that resolves to the application incoming viewing secret key.
   */
  getAppIncomingViewingSecretKey(account: AztecAddress, app: AztecAddress): Promise<Fr>;

  /**
   * Retrieves application outgoing viewing secret key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the application outgoing viewing secret key for.
   * @param app - The application address to retrieve the outgoing viewing secret key for.
   * @returns A Promise that resolves to the application outgoing viewing secret key.
   */
  getAppOutgoingViewingSecretKey(account: AztecAddress, app: AztecAddress): Promise<Fr>;

  /**
   * Retrieves the sk_m for the pk_m and a generator index of the key type.
   * @throws If the provided public key is not associated with any of the registered accounts.
   * @param masterPublicKey - The master public key to get secret key for.
   * @returns A Promise that resolves to sk_m.
   * @dev Used when feeding the sk_m to the kernel circuit for keys verification.
   */
  getMasterSecretKeyAndAppKeyGenerator(masterPublicKey: PublicKey): Promise<[GrumpkinPrivateKey, KeyGenerator]>;

  /**
   * Retrieves the master incoming viewing secret key (ivsk_m) corresponding to the specified master incoming viewing
   * public key (Ivpk_m).
   * @throws If the provided public key is not associated with any of the registered accounts.
   * @param masterIncomingViewingPublicKey - The master nullifier public key to get secret key for.
   * @returns A Promise that resolves to the master nullifier secret key.
   * @dev Used when feeding the master nullifier secret key to the kernel circuit for nullifier keys verification.
   */
  getMasterIncomingViewingSecretKeyForPublicKey(masterIncomingViewingPublicKey: PublicKey): Promise<GrumpkinPrivateKey>;

  /**
   * Gets the key validation request for a given master public key hash and contract address.
   * @throws If the account corresponding to the master public key hash does not exist in the key store.
   * @param pkMHash - The master public key hash.
   * @param contractAddress - The contract address to silo the secret key in the the key validation request with.
   * @returns The key validation request.
   */
  getKeyValidationRequest(pkMHash: Fr, contractAddress: AztecAddress): Promise<KeyValidationRequest>;

  /**
   * Rotates the master nullifier key for the specified account.
   *
   * @dev This function updates the secret and public keys associated with the account.
   * It appends a new secret key to the existing secret keys, derives the
   * corresponding public key, and updates the stored keys accordingly.
   *
   * @param account - The account address for which the master nullifier key is being rotated.
   * @param newSecretKey - (Optional) A new secret key of type Fq. If not provided, a random key is generated.
   * @throws If the account does not have existing nullifier secret keys or public keys.
   * @returns A Promise that resolves when the key rotation is complete.
   */
  rotateMasterNullifierKey(account: AztecAddress, secretKey: Fq): Promise<void>;
}
