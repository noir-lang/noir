import { AztecAddress, type GrumpkinScalar, type PublicKey } from '@aztec/circuits.js';
import { Aes128 } from '@aztec/circuits.js/barretenberg';

import { deriveAESSecret } from './encryption_utils.js';

/**
 * An encrypted log header, containing the address of the log along with utility
 * functions to compute and decrypt its ciphertext.
 *
 * Using AES-128-CBC for encryption.
 * Can be used for both incoming and outgoing logs.
 *
 */
export class EncryptedLogHeader {
  constructor(public readonly address: AztecAddress) {}

  /**
   * Serializes the log header to a buffer
   *
   * @returns The serialized log header
   */
  public toBuffer(): Buffer {
    return this.address.toBuffer();
  }

  public static fromBuffer(buf: Buffer): EncryptedLogHeader {
    return new EncryptedLogHeader(AztecAddress.fromBuffer(buf));
  }

  /**
   * Computes the ciphertext of the encrypted log header
   *
   * @param secret - An ephemeral secret key
   * @param publicKey - The incoming or outgoing viewing key of the "recipient" of this log
   * @returns The ciphertext of the encrypted log header
   */
  public computeCiphertext(secret: GrumpkinScalar, publicKey: PublicKey) {
    const aesSecret = deriveAESSecret(secret, publicKey);
    const key = aesSecret.subarray(0, 16);
    const iv = aesSecret.subarray(16, 32);

    const aes128 = new Aes128();
    const buffer = this.toBuffer();
    return aes128.encryptBufferCBC(buffer, iv, key);
  }

  /**
   *
   * @param ciphertext - The ciphertext buffer
   * @param secret - The private key matching the public key used in encryption
   * @param publicKey - The public key generated with the ephemeral secret key used in encryption
   *                    e.g., eph_sk * G
   * @returns
   */
  public static fromCiphertext(
    ciphertext: Buffer | bigint[],
    secret: GrumpkinScalar,
    publicKey: PublicKey,
  ): EncryptedLogHeader {
    const input = Buffer.isBuffer(ciphertext) ? ciphertext : Buffer.from(ciphertext.map((x: bigint) => Number(x)));

    const aesSecret = deriveAESSecret(secret, publicKey);
    const key = aesSecret.subarray(0, 16);
    const iv = aesSecret.subarray(16, 32);

    const aes128 = new Aes128();
    const buffer = aes128.decryptBufferCBC(input, iv, key);
    return EncryptedLogHeader.fromBuffer(buffer);
  }
}
