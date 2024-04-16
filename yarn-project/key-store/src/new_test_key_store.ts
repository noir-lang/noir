import { type NewKeyStore, type PublicKey } from '@aztec/circuit-types';
import {
  AztecAddress,
  Fr,
  GeneratorIndex,
  type GrumpkinPrivateKey,
  GrumpkinScalar,
  type PartialAddress,
  Point,
} from '@aztec/circuits.js';
import { type Grumpkin } from '@aztec/circuits.js/barretenberg';
import { poseidon2Hash, sha512ToGrumpkinScalar } from '@aztec/foundation/crypto';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';

/**
 * TestKeyStore is an implementation of the KeyStore interface, used for managing key pairs in a testing environment.
 * It should be utilized in testing scenarios where secure key management is not required, and ease-of-use is prioritized.
 */
export class NewTestKeyStore implements NewKeyStore {
  #keys: AztecMap<string, Buffer>;

  constructor(private curve: Grumpkin, database: AztecKVStore) {
    this.#keys = database.openMap('key_store');
  }

  /**
   * Creates a new account from a randomly generated secret key.
   * @returns A promise that resolves to the newly created account's AztecAddress.
   */
  public createAccount(): Promise<AztecAddress> {
    const sk = Fr.random();
    const partialAddress = Fr.random();
    return this.addAccount(sk, partialAddress);
  }

  /**
   * Adds an account to the key store from the provided secret key.
   * @param sk - The secret key of the account.
   * @param partialAddress - The partial address of the account.
   * @returns The account's address.
   */
  public async addAccount(sk: Fr, partialAddress: PartialAddress): Promise<AztecAddress> {
    // First we derive master secret keys -  we use sha512 here because this derivation will never take place
    // in a circuit
    const masterNullifierSecretKey = sha512ToGrumpkinScalar([sk, GeneratorIndex.NSK_M]);
    const masterIncomingViewingSecretKey = sha512ToGrumpkinScalar([sk, GeneratorIndex.IVSK_M]);
    const masterOutgoingViewingSecretKey = sha512ToGrumpkinScalar([sk, GeneratorIndex.OVSK_M]);
    const masterTaggingSecretKey = sha512ToGrumpkinScalar([sk, GeneratorIndex.TSK_M]);

    // Then we derive master public keys
    const masterNullifierPublicKey = this.curve.mul(this.curve.generator(), masterNullifierSecretKey);
    const masterIncomingViewingPublicKey = this.curve.mul(this.curve.generator(), masterIncomingViewingSecretKey);
    const masterOutgoingViewingPublicKey = this.curve.mul(this.curve.generator(), masterOutgoingViewingSecretKey);
    const masterTaggingPublicKey = this.curve.mul(this.curve.generator(), masterTaggingSecretKey);

    // We hash the public keys to get the public keys hash
    const publicKeysHash = poseidon2Hash([
      masterNullifierPublicKey,
      masterIncomingViewingPublicKey,
      masterOutgoingViewingPublicKey,
      masterTaggingPublicKey,
      GeneratorIndex.PUBLIC_KEYS_HASH,
    ]);

    // We hash the partial address and the public keys hash to get the account address
    // TODO(#5726): Should GeneratorIndex.CONTRACT_ADDRESS be removed given that we introduced CONTRACT_ADDRESS_V1?
    // TODO(#5726): Move the following line to AztecAddress class?
    const accountAddressFr = poseidon2Hash([partialAddress, publicKeysHash, GeneratorIndex.CONTRACT_ADDRESS_V1]);
    const accountAddress = AztecAddress.fromField(accountAddressFr);

    // We store all the public and secret keys in the database
    await this.#keys.set(`${accountAddress.toString()}-nsk_m`, masterNullifierSecretKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ivsk_m`, masterIncomingViewingSecretKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ovsk_m`, masterOutgoingViewingSecretKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-tsk_m`, masterTaggingSecretKey.toBuffer());

    await this.#keys.set(`${accountAddress.toString()}-npk_m`, masterNullifierPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ivpk_m`, masterIncomingViewingPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ovpk_m`, masterOutgoingViewingPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-tpk_m`, masterTaggingPublicKey.toBuffer());

    // At last, we return the newly derived account address
    return Promise.resolve(accountAddress);
  }

  /**
   * Retrieves addresses of accounts stored in the key store.
   * @returns A Promise that resolves to an array of account addresses.
   */
  public getAccounts(): Promise<AztecAddress[]> {
    const allMapKeys = Array.from(this.#keys.keys());
    // We return account addresses based on the map keys that end with '-nsk_m'
    const accounts = allMapKeys.filter(key => key.endsWith('-nsk_m')).map(key => key.split('-')[0]);
    return Promise.resolve(accounts.map(account => AztecAddress.fromString(account)));
  }

  /**
   * Gets the master nullifier public key for a given account.
   * @throws If the account does not exist in the key store.
   * @param account - The account address for which to retrieve the master nullifier public key.
   * @returns The master nullifier public key for the account.
   */
  public getMasterNullifierPublicKey(account: AztecAddress): Promise<PublicKey> {
    const masterNullifierPublicKeyBuffer = this.#keys.get(`${account.toString()}-npk_m`);
    if (!masterNullifierPublicKeyBuffer) {
      throw new Error(`Account ${account.toString()} does not exist.`);
    }
    return Promise.resolve(Point.fromBuffer(masterNullifierPublicKeyBuffer));
  }

  /**
   * Gets the master incoming viewing public key for a given account.
   * @throws If the account does not exist in the key store.
   * @param account - The account address for which to retrieve the master incoming viewing public key.
   * @returns The master incoming viewing public key for the account.
   */
  public getMasterIncomingViewingPublicKey(account: AztecAddress): Promise<PublicKey> {
    const masterIncomingViewingPublicKeyBuffer = this.#keys.get(`${account.toString()}-ivpk_m`);
    if (!masterIncomingViewingPublicKeyBuffer) {
      throw new Error(`Account ${account.toString()} does not exist.`);
    }
    return Promise.resolve(Point.fromBuffer(masterIncomingViewingPublicKeyBuffer));
  }

  /**
   * Retrieves the master outgoing viewing public key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the master outgoing viewing key for.
   * @returns A Promise that resolves to the master outgoing viewing key.
   */
  public getMasterOutgoingViewingPublicKey(account: AztecAddress): Promise<PublicKey> {
    const masterOutgoingViewingPublicKeyBuffer = this.#keys.get(`${account.toString()}-ovpk_m`);
    if (!masterOutgoingViewingPublicKeyBuffer) {
      throw new Error(`Account ${account.toString()} does not exist.`);
    }
    return Promise.resolve(Point.fromBuffer(masterOutgoingViewingPublicKeyBuffer));
  }

  /**
   * Retrieves the master tagging public key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the master tagging key for.
   * @returns A Promise that resolves to the master tagging key.
   */
  public getMasterTaggingPublicKey(account: AztecAddress): Promise<PublicKey> {
    const masterTaggingPublicKeyBuffer = this.#keys.get(`${account.toString()}-tpk_m`);
    if (!masterTaggingPublicKeyBuffer) {
      throw new Error(`Account ${account.toString()} does not exist.`);
    }
    return Promise.resolve(Point.fromBuffer(masterTaggingPublicKeyBuffer));
  }

  /**
   * Retrieves application nullifier secret key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the application nullifier secret key for.
   * @param app - The application address to retrieve the nullifier secret key for.
   * @returns A Promise that resolves to the application nullifier secret key.
   */
  public getAppNullifierSecretKey(account: AztecAddress, app: AztecAddress): Promise<Fr> {
    const masterNullifierSecretKeyBuffer = this.#keys.get(`${account.toString()}-nsk_m`);
    if (!masterNullifierSecretKeyBuffer) {
      throw new Error(`Account ${account.toString()} does not exist.`);
    }
    const masterNullifierSecretKey = GrumpkinScalar.fromBuffer(masterNullifierSecretKeyBuffer);

    return Promise.resolve(
      poseidon2Hash([masterNullifierSecretKey.high, masterNullifierSecretKey.low, app, GeneratorIndex.NSK_M]),
    );
  }

  /**
   * Retrieves application incoming viewing secret key.
   * @throws If the account does not exist in the key store.
   * @param account - The account to retrieve the application incoming viewing secret key for.
   * @param app - The application address to retrieve the incoming viewing secret key for.
   * @returns A Promise that resolves to the application incoming viewing secret key.
   */
  public getAppIncomingViewingSecretKey(account: AztecAddress, app: AztecAddress): Promise<Fr> {
    const masterIncomingViewingSecretKeyBuffer = this.#keys.get(`${account.toString()}-ivsk_m`);
    if (!masterIncomingViewingSecretKeyBuffer) {
      throw new Error(`Account ${account.toString()} does not exist.`);
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
  public getAppOutgoingViewingSecretKey(account: AztecAddress, app: AztecAddress): Promise<Fr> {
    const masterOutgoingViewingSecretKeyBuffer = this.#keys.get(`${account.toString()}-ovsk_m`);
    if (!masterOutgoingViewingSecretKeyBuffer) {
      throw new Error(`Account ${account.toString()} does not exist.`);
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
    // We iterate over the map keys to find the account address that corresponds to the provided public key
    for (const [key, value] of this.#keys.entries()) {
      if (value.equals(masterNullifierPublicKey.toBuffer())) {
        // We extract the account address from the map key
        const accountAddress = key.split('-')[0];
        // We fetch the secret key and return it
        const masterNullifierSecretKeyBuffer = this.#keys.get(`${accountAddress.toString()}-nsk_m`);
        if (!masterNullifierSecretKeyBuffer) {
          throw new Error(`Could not find master nullifier secret key for account ${accountAddress.toString()}`);
        }
        return Promise.resolve(GrumpkinScalar.fromBuffer(masterNullifierSecretKeyBuffer));
      }
    }

    throw new Error(`Could not find master nullifier secret key for public key ${masterNullifierPublicKey.toString()}`);
  }
}
