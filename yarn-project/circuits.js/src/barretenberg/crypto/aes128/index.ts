import { BarretenbergSync, RawBuffer } from '@aztec/bb.js';

import { Buffer } from 'buffer';

/**
 * AES-128-CBC encryption/decryption.
 */
export class Aes128 {
  /**
   * Encrypt a buffer using AES-128-CBC.
   * @param data - Data to encrypt.
   * @param iv - AES initialization vector.
   * @param key - Key to encrypt with.
   * @returns Encrypted data.
   */
  public encryptBufferCBC(data: Uint8Array, iv: Uint8Array, key: Uint8Array) {
    const rawLength = data.length;
    const numPaddingBytes = 16 - (rawLength % 16);
    const paddingBuffer = Buffer.alloc(numPaddingBytes);
    // input num bytes needs to be a multiple of 16 and at least 1 byte
    // node uses PKCS#7-Padding scheme, where padding byte value = the number of padding bytes
    paddingBuffer.fill(numPaddingBytes);
    const input = Buffer.concat([data, paddingBuffer]);

    const api = BarretenbergSync.getSingleton();
    return Buffer.from(
      api.aesEncryptBufferCbc(new RawBuffer(input), new RawBuffer(iv), new RawBuffer(key), input.length),
    );
  }

  /**
   * Decrypt a buffer using AES-128-CBC.
   * @param data - Data to decrypt.
   * @param iv - AES initialization vector.
   * @param key - Key to decrypt with.
   * @returns Decrypted data.
   */
  public decryptBufferCBC(data: Uint8Array, iv: Uint8Array, key: Uint8Array) {
    const api = BarretenbergSync.getSingleton();
    const paddedBuffer = Buffer.from(
      api.aesDecryptBufferCbc(new RawBuffer(data), new RawBuffer(iv), new RawBuffer(key), data.length),
    );
    const paddingToRemove = paddedBuffer[paddedBuffer.length - 1];
    return paddedBuffer.subarray(0, paddedBuffer.length - paddingToRemove);
  }
}
