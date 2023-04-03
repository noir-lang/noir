import aes from 'browserify-aes';
import { v4 } from 'uuid';
import { EthAddress } from '@aztec/foundation';
import { pbkdf2, scrypt, keccak256, randomBytes } from '../crypto/index.js';

interface ScryptKdfParams {
  dklen: number;
  n: number;
  p: number;
  r: number;
  salt: string;
}

interface PbKdf2Params {
  dklen: number;
  c: number;
  prf: string;
  salt: string;
}

export interface KeyStoreJson {
  address?: string;
  crypto: {
    cipher: string;
    ciphertext: string;
    cipherparams: {
      iv: string;
    };
    kdf: string;
    kdfparams: ScryptKdfParams | PbKdf2Params;
    mac: string;
  };
  id: string;
  version: number;
}

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

export interface KeyStoreEncryptOptions {
  cipher?: string;
  salt?: Buffer;
  iv?: Buffer;
  kdf?: 'scrypt' | 'pbkdf2';
  id?: string;
  c?: number;
  dklen?: number;
  n?: number;
  r?: number;
  p?: number;
}

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
