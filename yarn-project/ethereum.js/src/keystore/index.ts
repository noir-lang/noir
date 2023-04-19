import aes from 'browserify-aes';
import { v4 } from 'uuid';
import { EthAddress } from '@aztec/foundation';
import { pbkdf2, scrypt, keccak256, randomBytes } from '../crypto/index.js';

/**
 * Represents the Scrypt key derivation function parameters.
 * Provides a set of properties required for deriving cryptographic keys using the Scrypt algorithm.
 */
interface ScryptKdfParams {
  /**
   * The desired output key length in bytes.
   */
  dklen: number;
  /**
   * The cost factor for scrypt key derivation function.
   */
  n: number;
  /**
   * The parallelization factor for the scrypt key derivation function.
   */
  p: number;
  /**
   * The CPU/memory cost factor for scrypt key derivation.
   */
  r: number;
  /**
   * A cryptographic element used to enhance password security.
   */
  salt: string;
}

/**
 * Represents the PBKDF2 key derivation function parameters.
 * Provides the necessary information and options for the PBKDF2 algorithm to derive a cryptographic key from a password.
 */
interface PbKdf2Params {
  /**
   * The desired length of the derived key in bytes.
   */
  dklen: number;
  /**
   * The iteration count for the PBKDF2 key derivation function.
   */
  c: number;
  /**
   * Pseudorandom function (PRF) used for key derivation.
   */
  prf: string;
  /**
   * A random sequence of bytes used as an additional input for the key derivation function.
   */
  salt: string;
}

/**
 * Represents a keystore object in JSON format.
 * Contains necessary information required to encrypt and decrypt private keys using a user-provided password.
 */
export interface KeyStoreJson {
  /**
   * Ethereum address associated with the keystore.
   */
  address?: string;
  /**
   * Cryptographic configurations and encrypted data.
   */
  crypto: {
    /**
     * The encryption algorithm used to secure the private key.
     */
    cipher: string;
    /**
     * The encrypted private key in hexadecimal format.
     */
    ciphertext: string;
    /**
     * Parameters required for cipher initialization.
     */
    cipherparams: {
      /**
       * Initialization vector for the cipher algorithm.
       */
      iv: string;
    };
    /**
     * Key derivation function used for encryption.
     */
    kdf: string;
    /**
     * Key derivation function parameters for password-based key generation.
     */
    kdfparams: ScryptKdfParams | PbKdf2Params;
    /**
     * Message authentication code generated from encrypted data.
     */
    mac: string;
  };
  /**
   * Unique identifier for the keystore object.
   */
  id: string;
  /**
   * The version of the key store format.
   */
  version: number;
}

/**
 * Decrypt a private key from a V3 keystore JSON object using the provided password.
 * Supports 'scrypt' and 'pbkdf2' key derivation functions. Throws an error if the
 * password is incorrect, keystore format is invalid, or unsupported key derivation
 * or cipher schemes are used.
 *
 * @param v3Keystore - The V3 keystore JSON object containing encrypted private key information.
 * @param password - The password used for encryption/decryption of the private key.
 * @returns A Promise that resolves to the decrypted private key as a Buffer.
 */
export async function decryptFromKeyStoreJson(v3Keystore: KeyStoreJson, password: string): Promise<Buffer> {
  if (!password.length) {
    throw new Error('No password given.');
  }

  const json = v3Keystore;

  if (json.version !== 3) {
    throw new Error('Not a valid V3 wallet');
  }

  let derivedKey: Buffer;

  if (json.crypto.kdf === 'scrypt') {
    const { n, r, p, dklen, salt } = json.crypto.kdfparams as ScryptKdfParams;

    derivedKey = await scrypt(Buffer.from(password), Buffer.from(salt, 'hex'), n, r, p, dklen);
  } else if (json.crypto.kdf === 'pbkdf2') {
    const { prf, c, dklen, salt } = json.crypto.kdfparams as PbKdf2Params;

    if (prf !== 'hmac-sha256') {
      throw new Error('Unsupported parameters to PBKDF2');
    }

    derivedKey = await pbkdf2(Buffer.from(password), Buffer.from(salt, 'hex'), c, dklen);
  } else {
    throw new Error('Unsupported key derivation scheme');
  }

  const ciphertext = Buffer.from(json.crypto.ciphertext, 'hex');

  const mac = keccak256(Buffer.concat([derivedKey.slice(16, 32), ciphertext]));
  if (mac.toString('hex') !== json.crypto.mac) {
    throw new Error('Key derivation failed - possibly wrong password');
  }

  const iv = Buffer.from(json.crypto.cipherparams.iv, 'hex');
  const aesKey = derivedKey.slice(0, 16);

  const decipher = aes.createDecipheriv(json.crypto.cipher, aesKey, iv);
  return Buffer.concat([decipher.update(ciphertext), decipher.final()]);
}

/**
 * Represents the encryption options for a KeyStore JSON file.
 * Provides optional parameters to customize the encryption process, such as cipher algorithm, salt, iv, kdf, and other related attributes.
 */
export interface KeyStoreEncryptOptions {
  /**
   * Cipher algorithm used for encryption.
   */
  cipher?: string;
  /**
   * A random value used to ensure unique derived encryption keys.
   */
  salt?: Buffer;
  /**
   * Initialization Vector for the AES cipher.
   */
  iv?: Buffer;
  /**
   * Key derivation function used for encryption/decryption.
   */
  kdf?: 'scrypt' | 'pbkdf2';
  /**
   * Unique identifier for the key store.
   */
  id?: string;
  /**
   * The iteration count for the PBKDF2 key derivation function.
   */
  c?: number;
  /**
   * Length of the derived key in bytes.
   */
  dklen?: number;
  /**
   * The cost factor determining the CPU/memory complexity of the scrypt key derivation function.
   */
  n?: number;
  /**
   * The scrypt memory cost factor.
   */
  r?: number;
  /**
   * The parallelization factor for the scrypt key derivation function.
   */
  p?: number;
}

/**
 * Encrypts a private key to a KeyStore JSON object using the specified password and encryption options.
 * The resulting KeyStore JSON can be used to securely store the private key, and later decrypt it with the same password.
 * Supports 'scrypt' and 'pbkdf2' key derivation functions (KDF) for generating the derived key.
 * Uses AES-128-CTR cipher algorithm for encrypting the private key.
 * Throws an error if unsupported cipher or KDF is provided.
 *
 * @param privateKey - The private key Buffer to be encrypted.
 * @param address - The EthAddress associated with the privateKey.
 * @param password - The password string used for encryption.
 * @param options - Optional configuration settings for the encryption process.
 * @returns A Promise resolving to a KeyStoreJson object containing the encrypted private key and related information.
 */
export async function encryptToKeyStoreJson(
  privateKey: Buffer,
  address: EthAddress,
  password: string,
  options: KeyStoreEncryptOptions = {},
): Promise<KeyStoreJson> {
  const cipherAlgo = options.cipher || 'aes-128-ctr';
  const salt = options.salt ? options.salt : randomBytes(32);
  const iv = options.iv ? options.iv : randomBytes(16);
  const kdf = options.kdf || 'scrypt';
  const id = options.id || v4({ random: randomBytes(16) });

  if (cipherAlgo !== 'aes-128-ctr') {
    throw new Error('Unsupported cipher');
  }

  let derivedKey;
  let kdfparams;

  if (kdf === 'pbkdf2') {
    const { c = 262144, dklen = 32 } = options;
    derivedKey = await pbkdf2(Buffer.from(password), salt, c, dklen);
    kdfparams = { c, dklen, prf: 'hmac-sha256', salt: salt.toString('hex') };
  } else if (kdf === 'scrypt') {
    const { n = 8192, r = 8, p = 1, dklen = 32 } = options;

    derivedKey = await scrypt(Buffer.from(password), salt, n, r, p, dklen);
    kdfparams = { n, r, p, dklen, salt: salt.toString('hex') };
  } else {
    throw new Error('Unsupported kdf');
  }

  const aesKey = derivedKey.slice(0, 16);

  const cipher = aes.createCipheriv(cipherAlgo, aesKey, iv);
  if (!cipher) {
    throw new Error('Unsupported cipher');
  }

  const ciphertext = Buffer.concat([cipher.update(privateKey), cipher.final()]);

  const mac = keccak256(Buffer.concat([derivedKey.slice(16, 32), ciphertext]));

  return {
    version: 3,
    id,
    address: address.toString().toLowerCase().replace('0x', ''),
    crypto: {
      ciphertext: ciphertext.toString('hex'),
      cipherparams: {
        iv: iv.toString('hex'),
      },
      cipher: 'aes-128-ctr',
      kdf,
      kdfparams,
      mac: mac.toString('hex'),
    },
  };
}
