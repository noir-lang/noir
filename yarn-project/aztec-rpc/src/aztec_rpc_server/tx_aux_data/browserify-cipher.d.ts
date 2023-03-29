declare module 'browserify-cipher' {
  import { Cipher } from 'crypto';

  type CipherTypes = 'aes-128-cbc';

  interface CipherOptions {
    iv?: Buffer;
  }

  function createCipher(algorithm: CipherTypes, key: Buffer, options?: CipherOptions): Cipher;
  function createCipheriv(algorithm: CipherTypes, key: Buffer, iv: Buffer): Cipher;
  function createDecipher(algorithm: CipherTypes, key: Buffer, options?: CipherOptions): Cipher;
  function createDecipheriv(algorithm: CipherTypes, key: Buffer, iv: Buffer): Cipher;
  function getCiphers(): CipherTypes[];
}
