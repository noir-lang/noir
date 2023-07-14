import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';

/**
 * AES-128-CBC encryption/decryption.
 */
export class Aes128 {
  constructor(private wasm: IWasmModule) {}

  /**
   * Encrypt a buffer using AES-128-CBC.
   * @param data - Data to encrypt.
   * @param iv - AES initialisation vector.
   * @param key - Key to encrypt with.
   * @returns Encrypted data.
   */
  public encryptBufferCBC(data: Uint8Array, iv: Uint8Array, key: Uint8Array) {
    const rawLength = data.length;
    const numPaddingBytes = rawLength % 16 != 0 ? 16 - (rawLength % 16) : 0;
    const paddingBuffer = Buffer.alloc(numPaddingBytes);
    // input num bytes needs to be a multiple of 16
    // node uses PKCS#7-Padding scheme, where padding byte value = the number of padding bytes
    if (numPaddingBytes != 0) {
      paddingBuffer.fill(numPaddingBytes);
    }
    const input = Buffer.concat([data, paddingBuffer]);
    const mem = this.wasm.call('bbmalloc', input.length + key.length + iv.length + input.length);
    this.wasm.writeMemory(mem, input);
    this.wasm.writeMemory(mem + input.length, iv);
    this.wasm.writeMemory(mem + input.length + iv.length, key);
    this.wasm.call(
      'aes__encrypt_buffer_cbc',
      mem,
      mem + input.length,
      mem + input.length + iv.length,
      input.length,
      mem + input.length + iv.length + key.length,
    );
    const result: Buffer = Buffer.from(
      this.wasm.getMemorySlice(
        mem + input.length + key.length + iv.length,
        mem + input.length + key.length + iv.length + input.length,
      ),
    );
    this.wasm.call('bbfree', mem);
    return result;
  }

  /**
   * Decrypt a buffer using AES-128-CBC.
   * @param data - Data to decrypt.
   * @param iv - AES initialisation vector.
   * @param key - Key to decrypt with.
   * @returns Decrypted data.
   */
  public decryptBufferCBC(data: Uint8Array, iv: Uint8Array, key: Uint8Array) {
    const mem = this.wasm.call('bbmalloc', data.length + key.length + iv.length + data.length);
    this.wasm.writeMemory(mem, data);
    this.wasm.writeMemory(mem + data.length, iv);
    this.wasm.writeMemory(mem + data.length + iv.length, key);
    this.wasm.call(
      'aes__decrypt_buffer_cbc',
      mem,
      mem + data.length,
      mem + data.length + iv.length,
      data.length,
      mem + data.length + iv.length + key.length,
    );
    const result: Buffer = Buffer.from(
      this.wasm.getMemorySlice(
        mem + data.length + key.length + iv.length,
        mem + data.length + key.length + iv.length + data.length,
      ),
    );
    this.wasm.call('bbfree', mem);
    return result;
  }
}
