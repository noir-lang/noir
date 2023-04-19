import { EthAddress } from '@aztec/foundation';
import { mnemonicToSeedSync } from 'bip39';
import hdkey from 'hdkey';
import { default as elliptic } from 'elliptic';
import { keccak256, randomBytes } from '../crypto/index.js';
import { decryptFromKeyStoreJson, encryptToKeyStoreJson, KeyStoreJson } from '../keystore/index.js';
import { EthSignature, hashMessage, recoverFromSignature, signMessage } from '../eth_sign/index.js';
import { EthTransaction, signedTransaction, signTransaction } from '../eth_transaction/index.js';
import { TypedData } from '../eth_typed_data/typed_data.js';
import { getTypedDataHash } from '../eth_typed_data/index.js';

const secp256k1 = new elliptic.ec('secp256k1');

/**
 * The EthAccount class represents an Ethereum account with associated private and public keys
 * and provides methods for creating accounts, signing transactions, messages, and typed data.
 * It also supports operations like validating signatures, generating KeyStore JSON, and creating
 * accounts from mnemonics or seeds using HD wallet derivation paths.
 */
export class EthAccount {
  /**
   * The Ethereum address associated with the account.
   */
  public readonly address: EthAddress;
  /**
   * The public key associated with the Ethereum account.
   */
  public readonly publicKey: Buffer;

  constructor(
    /**
     * The private key of the Ethereum account.
     */
    public readonly privateKey: Buffer,
  ) {
    const ecKey = secp256k1.keyFromPrivate(privateKey);
    this.publicKey = Buffer.from(ecKey.getPublic(false, 'hex'), 'hex');
    // Why discarding first byte?
    const publicHash = keccak256(this.publicKey.slice(1));
    this.address = new EthAddress(publicHash.slice(-20));
  }

  /**
   * Create a new EthAccount instance using optional entropy.
   * This function generates a random private key, optionally combined with the provided entropy,
   * and creates an EthAccount instance with the corresponding address and public key.
   * Throws an error if the generated private key is invalid.
   *
   * @param entropy - Optional buffer containing entropy to be combined with the randomly generated private key. Default is a 32-byte random buffer.
   * @returns A new EthAccount instance with the generated private key, address, and public key.
   */
  public static create(entropy: Buffer = randomBytes(32)) {
    const innerHex = keccak256(Buffer.concat([randomBytes(32), entropy]));
    const middleHex = Buffer.concat([randomBytes(32), innerHex, randomBytes(32)]);
    const outerHex = keccak256(middleHex);
    return new EthAccount(outerHex);
  }

  /**
   * Creates an EthAccount instance from a mnemonic phrase and a derivation path.
   * The mnemonic is used to generate the seed, which is then used with the provided derivation path
   * to derive the private key for the account. This function is useful when working with
   * Hierarchical Deterministic (HD) wallets.
   *
   * @param mnemonic - The mnemonic phrase representing the seed for the HD wallet.
   * @param derivationPath - The derivation path to generate the EthAccount's private key.
   * @returns An EthAccount instance with the derived private key.
   */
  public static fromMnemonicAndPath(mnemonic: string, derivationPath: string) {
    const seed = mnemonicToSeedSync(mnemonic);
    return EthAccount.fromSeedAndPath(seed, derivationPath);
  }

  /**
   * Create an EthAccount instance from a seed and derivation path.
   * The function takes a Buffer containing the seed and a string with the derivation path,
   * and generates the corresponding private key by following the BIP32 HD wallet standard.
   * It then creates and returns an EthAccount object using the derived private key.
   *
   * @param seed - A Buffer containing the seed for the HD wallet.
   * @param derivationPath - A string representing the BIP32 derivation path.
   * @returns An EthAccount instance with the derived private key.
   */
  public static fromSeedAndPath(seed: Buffer, derivationPath: string) {
    const root = hdkey.fromMasterSeed(seed);
    const addrNode = root.derive(derivationPath);
    const privateKey = addrNode.privateKey;
    return new EthAccount(privateKey);
  }

  /**
   * Create an EthAccount instance from a KeyStoreJson object.
   * This method decrypts the encrypted private key in the v3 keystore with the provided password,
   * and initializes the EthAccount instance with the decrypted private key.
   * Throws an error if the password is incorrect or the decryption process fails.
   *
   * @param v3Keystore - The KeyStoreJson object representing the Ethereum keystore (v3 format).
   * @param password - The password used to encrypt the private key in the keystore.
   * @returns A Promise that resolves to an EthAccount instance.
   */
  public static async fromKeyStoreJson(v3Keystore: KeyStoreJson, password: string) {
    return new EthAccount(await decryptFromKeyStoreJson(v3Keystore, password));
  }

  /**
   * Sign an Ethereum transaction using the account's private key.
   * This method generates a digital signature for the provided transaction object
   * by leveraging the private key of the account instance. The signed transaction can then
   * be broadcasted to the Ethereum network for execution.
   *
   * @param tx - The EthTransaction object representing the details of the transaction to be signed.
   * @returns An EthSignature object containing the generated signature for the transaction.
   */
  public signTransaction(tx: EthTransaction) {
    return signTransaction(tx, this.privateKey);
  }

  /**
   * Checks if a signed Ethereum transaction matches the expected address.
   * The function takes an unsigned Ethereum transaction and its corresponding signature,
   * then recovers the signer's Ethereum address from the signature and compares it
   * to the current EthAccount instance's address.
   *
   * @param tx - The unsigned Ethereum transaction object.
   * @param signature - The Ethereum signature object for the given transaction.
   * @returns A boolean indicating whether the recovered address matches the EthAccount's address.
   */
  public signedTransaction(tx: EthTransaction, signature: EthSignature) {
    return signedTransaction(tx, signature).equals(this.address);
  }

  /**
   * Prefixes the arbitrary length message with the 'x19Ethereum Signed Message:n' preamble, and signs the message.
   * @returns An EthSignature instance with the signature components (r, s, v).
   */
  public signMessage(message: Buffer) {
    return signMessage(hashMessage(message), this.privateKey);
  }

  /**
   * Signs a 32 byte digest.
   * @returns An EthSignature instance with the signature components (r, s, v).
   */
  public signDigest(digest: Buffer) {
    if (digest.length !== 32) {
      throw new Error('Expected digest to be 32 bytes.');
    }
    return signMessage(digest, this.privateKey);
  }

  /**
   * Sign the typed data by first getting its hashed digest using the provided 'data' parameter,
   * and then signing the resulting 32-byte digest with the account's private key.
   * This function is useful for signing structured data according to EIP-712 standard.
   *
   * @param data - A TypedData object containing the type information and values to be signed.
   * @returns An EthSignature representing the signature of the hashed typed data.
   */
  public signTypedData(data: TypedData) {
    return this.signDigest(getTypedDataHash(data));
  }

  /**
   * Verifies if the given signature corresponds to the message signed by this EthAccount instance.
   * It hashes the input message with the Ethereum Signed Message preamble, recovers the signer's address
   * from the signature, and compares it against the EthAccount's address.
   *
   * @param message - The Buffer containing the message to be verified.
   * @param signature - The EthSignature instance representing the signature of the message.
   * @returns A boolean value indicating whether the signature corresponds to the message signed by this EthAccount.
   */
  public signedMessage(message: Buffer, signature: EthSignature) {
    return recoverFromSignature(hashMessage(message), signature).equals(this.address);
  }

  /**
   * Encrypts the EthAccount's private key into a KeyStore JSON format using the provided password.
   * The KeyStore JSON can be stored securely and used later to recreate the EthAccount instance.
   * The optional 'options' parameter can be used to customize the encryption process.
   *
   * @param password - The password used for encrypting the private key.
   * @param options - Optional configuration object for the encryption process.
   * @returns A KeyStoreJson instance representing the encrypted private key.
   */
  public toKeyStoreJson(password: string, options?: any) {
    return encryptToKeyStoreJson(this.privateKey, this.address, password, options);
  }
}
