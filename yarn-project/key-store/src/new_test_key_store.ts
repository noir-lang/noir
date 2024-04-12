import { type NewKeyStore, type PublicKey } from '@aztec/circuit-types';
import { AztecAddress, Fr, GeneratorIndex, type PartialAddress, Point } from '@aztec/circuits.js';
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
    const publicKeysHash = poseidon2Hash(
      [
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
      ],
      GeneratorIndex.PUBLIC_KEYS_HASH,
    );

    // We hash the partial address and the public keys hash to get the account address
    // TODO(#5726): Should GeneratorIndex.CONTRACT_ADDRESS be removed given that we introduced CONTRACT_ADDRESS_V1?
    // TODO(#5726): Move the following line to AztecAddress class?
    const accountAddressFr = poseidon2Hash([partialAddress, publicKeysHash], GeneratorIndex.CONTRACT_ADDRESS_V1);
    const accountAddress = AztecAddress.fromField(accountAddressFr);

    // We store the keys in the database
    await this.#keys.set(`${accountAddress.toString()}-npk_m`, masterNullifierPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ivpk_m`, masterIncomingViewingPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-ovpk_m`, masterOutgoingViewingPublicKey.toBuffer());
    await this.#keys.set(`${accountAddress.toString()}-tpk_m`, masterTaggingPublicKey.toBuffer());

    // At last, we return the newly derived account address
    return Promise.resolve(accountAddress);
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
}
