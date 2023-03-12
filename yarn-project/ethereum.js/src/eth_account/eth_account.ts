import { EthAddress } from '../eth_address/index.js';
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

export class EthAccount {
  public readonly address: EthAddress;
  public readonly publicKey: Buffer;

  constructor(public readonly privateKey: Buffer) {
    const ecKey = secp256k1.keyFromPrivate(privateKey);
    this.publicKey = Buffer.from(ecKey.getPublic(false, 'hex'), 'hex');
    // Why discarding first byte?
    const publicHash = keccak256(this.publicKey.slice(1));
    this.address = new EthAddress(publicHash.slice(-20));
  }

  public static create(entropy: Buffer = randomBytes(32)) {
    const innerHex = keccak256(Buffer.concat([randomBytes(32), entropy]));
    const middleHex = Buffer.concat([randomBytes(32), innerHex, randomBytes(32)]);
    const outerHex = keccak256(middleHex);
    return new EthAccount(outerHex);
  }

  public static fromMnemonicAndPath(mnemonic: string, derivationPath: string) {
    const seed = mnemonicToSeedSync(mnemonic);
    return EthAccount.fromSeedAndPath(seed, derivationPath);
  }

  public static fromSeedAndPath(seed: Buffer, derivationPath: string) {
    const root = hdkey.fromMasterSeed(seed);
    const addrNode = root.derive(derivationPath);
    const privateKey = addrNode.privateKey;
    return new EthAccount(privateKey);
  }

  public static async fromKeyStoreJson(v3Keystore: KeyStoreJson, password: string) {
    return new EthAccount(await decryptFromKeyStoreJson(v3Keystore, password));
  }

  public signTransaction(tx: EthTransaction) {
    return signTransaction(tx, this.privateKey);
  }

  public signedTransaction(tx: EthTransaction, signature: EthSignature) {
    return signedTransaction(tx, signature).equals(this.address);
  }

  /**
   * Prefixes the arbitrary length message with the '\x19Ethereum Signed Message:\n' preamble, and signs the message.
   */
  public signMessage(message: Buffer) {
    return signMessage(hashMessage(message), this.privateKey);
  }

  /**
   * Signs a 32 byte digest.
   */
  public signDigest(digest: Buffer) {
    if (digest.length !== 32) {
      throw new Error('Expected digest to be 32 bytes.');
    }
    return signMessage(digest, this.privateKey);
  }

  public signTypedData(data: TypedData) {
    return this.signDigest(getTypedDataHash(data));
  }

  public signedMessage(message: Buffer, signature: EthSignature) {
    return recoverFromSignature(hashMessage(message), signature).equals(this.address);
  }

  public toKeyStoreJson(password: string, options?: any) {
    return encryptToKeyStoreJson(this.privateKey, this.address, password, options);
  }
}
