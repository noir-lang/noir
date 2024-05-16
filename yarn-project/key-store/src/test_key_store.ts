import { type KeyStore, type PublicKey } from '@aztec/circuit-types';
import {
  AztecAddress,
  CompleteAddress,
  Fq,
  Fr,
  GeneratorIndex,
  type GrumpkinPrivateKey,
  GrumpkinScalar,
  type PartialAddress,
  Point,
  computeAddress,
  computeAppNullifierSecretKey,
  deriveKeys,
  derivePublicKeyFromSecretKey,
} from '@aztec/circuits.js';
import { poseidon2Hash } from '@aztec/foundation/crypto';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';

/**
 * TestKeyStore is an implementation of the KeyStore interface, used for managing key pairs in a testing environment.
 * It should be utilized in testing scenarios where secure key management is not required, and ease-of-use is prioritized.
 */
export class TestKeyStore implements KeyStore {
  #keys: AztecMap<string, Buffer>;

  constructor(database: AztecKVStore) {
    this.#keys = database.openMap('key_store');
  }

  /**
   * Creates a new account from a randomly generated secret key.
   * @returns A promise that resolves to the newly created account's CompleteAddress.
   */
  public createAccount(): Promise<CompleteAddress> {
    const sk = Fr.random();
    const partialAddress = Fr.random();
    return this.addAccount(sk, partialAddress);
  }

  /**
   * Adds an account to the key store from the provided secret key.
   * @param sk - The secret key of the account.
   * @param partialAddress - The partial address of the account.
   * @returns The account's complete address.
   */
  public async addAccount(sk: Fr, partialAddress: PartialAddress): Promise<CompleteAddress> {
    const {
      masterNullifierSecretKey,
      masterIncomingViewingSecretKey,
      masterOutgoingViewingSecretKey,
      masterTaggingSecretKey,
      publicKeys,
    } = deriveKeys(sk);

    const publicKeysHash = publicKeys.hash();
    const accountAddress = computeAddress(publicKeysHash, partialAddress);

    // We save the keys to db associated with the account address
    await this.#keys.set(`${accountAddress.toString()}-public_keys_hash`, publicKeysHash.toBuffer());

    // Naming of keys is as follows ${from}-${to}_m
    await this.#keys.set(`${accountAddress.toString()}-ivsk_m`, masterIncomingViewingSecretKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ovsk_m`, masterOutgoingViewingSecretKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-tsk_m`, masterTaggingSecretKey.toBuffer());
    // The key of the following is different from the others because the buffer can store multiple keys
    await this.#keys.set(`${accountAddress.toString()}-ns_keys_m`, masterNullifierSecretKey.toBuffer());

    await this.#keys.set(`${accountAddress.toString()}-np_keys_m`, publicKeys.masterNullifierPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ivpk_m`, publicKeys.masterIncomingViewingPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ovpk_m`, publicKeys.masterOutgoingViewingPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-tpk_m`, publicKeys.masterTaggingPublicKey.toBuffer());

    // We store a npk_m_hash-account_address map to make address easy to obtain with the hash later on
    await this.#keys.set(
      `${publicKeys.masterNullifierPublicKey.hash().toString()}-npk_m_hash`,
      accountAddress.toBuffer(),
    );

    // At last, we return the newly derived account address
    return Promise.resolve(new CompleteAddress(accountAddress, publicKeys, partialAddress));
  }

  /**
   * Retrieves addresses of accounts stored in the key store.
   * @returns A Promise that resolves to an array of account addresses.
   */
  public getAccounts(): Promise<AztecAddress[]> {
    const allMapKeys = Array.from(this.#keys.keys());
    // We return account addresses based on the map keys that end with '-ivsk_m'
    const accounts = allMapKeys.filter(key => key.endsWith('-ivsk_m')).map(key => key.split('-')[0]);
    return Promise.resolve(accounts.map(account => AztecAddress.fromString(account)));
  }

  /**
   * Gets the master nullifier public key for a given master nullifier public key hash.
   * @throws If the account corresponding to the master nullifier public key hash does not exist in the key store.
   * @param npkMHash - The master nullifier public key hash.
   * @returns The master nullifier public key for the account.
   */
  public getMasterNullifierPublicKey(npkMHash: Fr): Promise<PublicKey> {
    // Get the address for npk_m_hash
    const accountAddressBuffer = this.#keys.get(`${npkMHash.toString()}-npk_m_hash`);
    if (!accountAddressBuffer) {
      throw new Error(`Could no find address for master nullifier public key hash ${npkMHash}.`);
    }
    const accountAddress = AztecAddress.fromBuffer(accountAddressBuffer);

    // Get the master nullifier public keys buffer for the account
    const masterNullifierPublicKeysBuffer = this.#keys.get(`${accountAddress.toString()}-np_keys_m`);
    if (!masterNullifierPublicKeysBuffer) {
      throw new Error(
        `Could not find master nullifier public key for account ${accountAddress.toString()} whose address was successfully obtained with npk_m_hash ${npkMHash.toString()}.`,
      );
    }

    // We check that the buffer's length is a multiple of Point.SIZE_IN_BYTES
    if (masterNullifierPublicKeysBuffer.byteLength % Point.SIZE_IN_BYTES !== 0) {
      throw new Error("Master nullifier public key buffer's length is not a multiple of Point.SIZE_IN_BYTES.");
    }

    // Now we iterate over the public keys in the buffer to find the one that matches the hash
    const numKeys = masterNullifierPublicKeysBuffer.byteLength / Point.SIZE_IN_BYTES;
    for (let i = 0; i < numKeys; i++) {
      const masterNullifierPublicKey = Point.fromBuffer(
        masterNullifierPublicKeysBuffer.subarray(i * Point.SIZE_IN_BYTES, (i + 1) * Point.SIZE_IN_BYTES),
      );
      if (masterNullifierPublicKey.hash().equals(npkMHash)) {
        return Promise.resolve(masterNullifierPublicKey);
      }
    }
    throw new Error(`Could not find master nullifier public key for npk_m_hash ${npkMHash.toString()}.`);
  }

  /**
   * Gets the master incoming viewing public key for a given account.
   * @throws If the account does not exist in the key store.
   * @param account - The account address for which to retrieve the master incoming viewing public key.
   * @returns The master incoming viewing public key for the account.
   */
  public async getMasterIncomingViewingPublicKey(account: AztecAddress): Promise<PublicKey> {
    const masterIncomingViewingPublicKeyBuffer = this.#keys.get(`${account.toString()}-ivpk_m`);
    if (!masterIncomingViewingPublicKeyBuffer) {
      throw new Error(
        `Account ${account.toString()} does not exist. Registered accounts: ${await this.getAccounts()}.`,
      );
    }
    return Promise.resolve(Point.fromBuffer(masterIncomingViewingPublicKeyBuffer));
  }

  /**
   * Retrieves the master outgoing viewing public key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the master outgoing viewing key for.
   * @returns A Promise that resolves to the master outgoing viewing key.
   */
  public async getMasterOutgoingViewingPublicKey(account: AztecAddress): Promise<PublicKey> {
    const masterOutgoingViewingPublicKeyBuffer = this.#keys.get(`${account.toString()}-ovpk_m`);
    if (!masterOutgoingViewingPublicKeyBuffer) {
      throw new Error(
        `Account ${account.toString()} does not exist. Registered accounts: ${await this.getAccounts()}.`,
      );
    }
    return Promise.resolve(Point.fromBuffer(masterOutgoingViewingPublicKeyBuffer));
  }

  /**
   * Retrieves the master tagging public key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the master tagging key for.
   * @returns A Promise that resolves to the master tagging key.
   */
  public async getMasterTaggingPublicKey(account: AztecAddress): Promise<PublicKey> {
    const masterTaggingPublicKeyBuffer = this.#keys.get(`${account.toString()}-tpk_m`);
    if (!masterTaggingPublicKeyBuffer) {
      throw new Error(
        `Account ${account.toString()} does not exist. Registered accounts: ${await this.getAccounts()}.`,
      );
    }
    return Promise.resolve(Point.fromBuffer(masterTaggingPublicKeyBuffer));
  }

  /**
   * Derives and returns the application nullifier secret key for a given master nullifier public key hash.
   * @throws If the account corresponding to the master nullifier public key hash does not exist in the key store.
   * @param npkMHash - The master nullifier public key hash.
   * @param app - The application address to retrieve the nullifier secret key for.
   * @returns A Promise that resolves to the application nullifier secret key.
   */
  public getAppNullifierSecretKey(npkMHash: Fr, app: AztecAddress): Promise<Fr> {
    // First we get the account address for npk_m_hash
    const accountAddressBuffer = this.#keys.get(`${npkMHash.toString()}-npk_m_hash`);
    if (!accountAddressBuffer) {
      throw new Error(`Could no find address for master nullifier public key hash ${npkMHash}.`);
    }

    // Now we get the master nullifier secret keys for the account
    const masterNullifierSecretKeysBuffer = this.#keys.get(
      `${AztecAddress.fromBuffer(accountAddressBuffer).toString()}-ns_keys_m`,
    );
    if (!masterNullifierSecretKeysBuffer) {
      throw new Error(
        `Could not find master nullifier secret keys for account ${AztecAddress.fromBuffer(
          accountAddressBuffer,
        ).toString()}`,
      );
    }

    // Now we iterate over all the secret keys to find the one that matches the hash
    const numKeys = masterNullifierSecretKeysBuffer.byteLength / GrumpkinScalar.SIZE_IN_BYTES;
    for (let i = 0; i < numKeys; i++) {
      const secretKey = GrumpkinScalar.fromBuffer(
        masterNullifierSecretKeysBuffer.subarray(
          i * GrumpkinScalar.SIZE_IN_BYTES,
          (i + 1) * GrumpkinScalar.SIZE_IN_BYTES,
        ),
      );
      const publicKey = derivePublicKeyFromSecretKey(secretKey);
      if (publicKey.hash().equals(npkMHash)) {
        return Promise.resolve(computeAppNullifierSecretKey(secretKey, app));
      }
    }

    throw new Error(`Could not find master nullifier secret key for npk_m_hash ${npkMHash.toString()}.`);
  }

  /**
   * Retrieves application incoming viewing secret key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the application incoming viewing secret key for.
   * @param app - The application address to retrieve the incoming viewing secret key for.
   * @returns A Promise that resolves to the application incoming viewing secret key.
   */
  public async getAppIncomingViewingSecretKey(account: AztecAddress, app: AztecAddress): Promise<Fr> {
    const masterIncomingViewingSecretKeyBuffer = this.#keys.get(`${account.toString()}-ivsk_m`);
    if (!masterIncomingViewingSecretKeyBuffer) {
      throw new Error(
        `Account ${account.toString()} does not exist. Registered accounts: ${await this.getAccounts()}.`,
      );
    }
    const masterIncomingViewingSecretKey = GrumpkinScalar.fromBuffer(masterIncomingViewingSecretKeyBuffer);

    return Promise.resolve(
      poseidon2Hash([
        masterIncomingViewingSecretKey.high,
        masterIncomingViewingSecretKey.low,
        app,
        GeneratorIndex.IVSK_M,
      ]),
    );
  }

  /**
   * Retrieves application outgoing viewing secret key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the application outgoing viewing secret key for.
   * @param app - The application address to retrieve the outgoing viewing secret key for.
   * @returns A Promise that resolves to the application outgoing viewing secret key.
   */
  public async getAppOutgoingViewingSecretKey(account: AztecAddress, app: AztecAddress): Promise<Fr> {
    const masterOutgoingViewingSecretKeyBuffer = this.#keys.get(`${account.toString()}-ovsk_m`);
    if (!masterOutgoingViewingSecretKeyBuffer) {
      throw new Error(
        `Account ${account.toString()} does not exist. Registered accounts: ${await this.getAccounts()}.`,
      );
    }
    const masterOutgoingViewingSecretKey = GrumpkinScalar.fromBuffer(masterOutgoingViewingSecretKeyBuffer);

    return Promise.resolve(
      poseidon2Hash([
        masterOutgoingViewingSecretKey.high,
        masterOutgoingViewingSecretKey.low,
        app,
        GeneratorIndex.OVSK_M,
      ]),
    );
  }

  /**
   * Retrieves the master nullifier secret key (nsk_m) corresponding to the specified master nullifier public key
   * (Npk_m).
   * @throws If the provided public key is not associated with any of the registered accounts.
   * @param masterNullifierPublicKey - The master nullifier public key to get secret key for.
   * @returns A Promise that resolves to the master nullifier secret key.
   * @dev Used when feeding the master nullifier secret key to the kernel circuit for nullifier keys verification.
   */
  public getMasterNullifierSecretKeyForPublicKey(masterNullifierPublicKey: PublicKey): Promise<GrumpkinPrivateKey> {
    // We get the account address associated with the master nullifier public key hash
    const accountAddressBuffer = this.#keys.get(`${masterNullifierPublicKey.hash().toString()}-npk_m_hash`);
    if (!accountAddressBuffer) {
      throw new Error(
        `Could not find account address for master nullifier public key ${masterNullifierPublicKey.toString()}`,
      );
    }
    const accountAddress = AztecAddress.fromBuffer(accountAddressBuffer);

    // We fetch the public keys and find this specific public key's position in the buffer
    const masterNullifierPublicKeysBuffer = this.#keys.get(`${accountAddress.toString()}-np_keys_m`);
    if (!masterNullifierPublicKeysBuffer) {
      throw new Error(`Could not find master nullifier public keys for account ${accountAddress.toString()}`);
    }

    // We check that the buffer's length is a multiple of Point.SIZE_IN_BYTES
    if (masterNullifierPublicKeysBuffer.byteLength % Point.SIZE_IN_BYTES !== 0) {
      throw new Error("Master nullifier public key buffer's length is not a multiple of Point.SIZE_IN_BYTES.");
    }

    // Now we iterate over the public keys in the buffer to find the one that matches the hash
    const numKeys = masterNullifierPublicKeysBuffer.byteLength / Point.SIZE_IN_BYTES;
    let keyIndex = -1;
    for (let i = 0; i < numKeys; i++) {
      const publicKey = Point.fromBuffer(
        masterNullifierPublicKeysBuffer.subarray(i * Point.SIZE_IN_BYTES, (i + 1) * Point.SIZE_IN_BYTES),
      );
      if (publicKey.equals(masterNullifierPublicKey)) {
        keyIndex = i;
        break;
      }
    }

    // Now we fetch the secret keys buffer and extract the secret key at the same index
    const masterNullifierSecretKeysBuffer = this.#keys.get(`${accountAddress.toString()}-ns_keys_m`);
    if (!masterNullifierSecretKeysBuffer) {
      throw new Error(`Could not find master nullifier secret keys for account ${accountAddress.toString()}`);
    }

    // We extract the secret key from the buffer
    const secretKeyBuffer = masterNullifierSecretKeysBuffer.subarray(
      keyIndex * GrumpkinScalar.SIZE_IN_BYTES,
      (keyIndex + 1) * GrumpkinScalar.SIZE_IN_BYTES,
    );
    const secretKey = GrumpkinScalar.fromBuffer(secretKeyBuffer);

    // We sanity check that it's possible to derive the public key from the secret key
    if (!derivePublicKeyFromSecretKey(secretKey).equals(masterNullifierPublicKey)) {
      throw new Error(
        `Could not find master nullifier secret key for public key ${masterNullifierPublicKey.toString()}`,
      );
    }

    return Promise.resolve(secretKey);
  }

  /**
   * Retrieves the master incoming viewing secret key (ivsk_m) corresponding to the specified master incoming viewing
   * public key (Ivpk_m).
   * @throws If the provided public key is not associated with any of the registered accounts.
   * @param masterIncomingViewingPublicKey - The master nullifier public key to get secret key for.
   * @returns A Promise that resolves to the master nullifier secret key.
   * @dev Used when feeding the master nullifier secret key to the kernel circuit for nullifier keys verification.
   */
  public getMasterIncomingViewingSecretKeyForPublicKey(
    masterIncomingViewingPublicKey: PublicKey,
  ): Promise<GrumpkinPrivateKey> {
    // We iterate over the map keys to find the account address that corresponds to the provided public key
    for (const [key, value] of this.#keys.entries()) {
      if (value.equals(masterIncomingViewingPublicKey.toBuffer())) {
        // We extract the account address from the map key
        const accountAddress = key.split('-')[0];
        // We fetch the secret key and return it
        const masterIncomingViewingSecretKeyBuffer = this.#keys.get(`${accountAddress.toString()}-ivsk_m`);
        if (!masterIncomingViewingSecretKeyBuffer) {
          throw new Error(`Could not find master incoming viewing secret key for account ${accountAddress.toString()}`);
        }
        return Promise.resolve(GrumpkinScalar.fromBuffer(masterIncomingViewingSecretKeyBuffer));
      }
    }

    throw new Error(
      `Could not find master incoming viewing secret key for public key ${masterIncomingViewingPublicKey.toString()}`,
    );
  }

  /**
   * Retrieves public keys hash of the account
   * @throws If the provided account address is not associated with any of the registered accounts.
   * @param account - The account address to get public keys hash for.
   * @returns A Promise that resolves to the public keys hash.
   */
  public async getPublicKeysHash(account: AztecAddress): Promise<Fr> {
    const publicKeysHashBuffer = this.#keys.get(`${account.toString()}-public_keys_hash`);
    if (!publicKeysHashBuffer) {
      throw new Error(
        `Account ${account.toString()} does not exist. Registered accounts: ${await this.getAccounts()}.`,
      );
    }
    return Promise.resolve(Fr.fromBuffer(publicKeysHashBuffer));
  }

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
  public async rotateMasterNullifierKey(account: AztecAddress, newSecretKey: Fq = Fq.random()) {
    // We append the secret key to the original secret key
    const secretKeysBuffer = this.#keys.get(`${account.toString()}-ns_keys_m`);
    if (!secretKeysBuffer) {
      throw new Error(`Could not find nullifier secret keys for account ${account.toString()}`);
    }

    // We append the new secret key to the buffer of secret keys
    const newSecretKeysBuffer = Buffer.concat([secretKeysBuffer, newSecretKey.toBuffer()]);
    await this.#keys.set(`${account.toString()}-ns_keys_m`, newSecretKeysBuffer);

    // Now we derive the public key from the new secret key and append it to the buffer of original public keys
    const newPublicKey = derivePublicKeyFromSecretKey(newSecretKey);
    const publicKeysBuffer = this.#keys.get(`${account.toString()}-np_keys_m`);
    if (!publicKeysBuffer) {
      throw new Error(`Could not find nullifier public keys for account ${account.toString()}`);
    }

    // We append the new public key to the buffer of public keys
    const newPublicKeysBuffer = Buffer.concat([publicKeysBuffer, newPublicKey.toBuffer()]);
    await this.#keys.set(`${account.toString()}-np_keys_m`, newPublicKeysBuffer);

    // We store a npk_m_hash-account_address map to make address easy to obtain with the hash later on
    await this.#keys.set(`${newPublicKey.hash().toString()}-npk_m_hash`, account.toBuffer());
  }
}
