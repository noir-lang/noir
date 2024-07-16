import { type GrumpkinScalar, type PublicKey } from '@aztec/circuits.js';
import { Aes128 } from '@aztec/circuits.js/barretenberg';

import { deriveAESSecret } from '../encryption_utils.js';

export abstract class EncryptedLogIncomingBody {
  public abstract toBuffer(): Buffer;

  /**
   * Decrypts a log body
   *
   * @param ciphertext - The ciphertext buffer
   * @param ivskOrEphSk - The private key matching the public key used in encryption (the viewing secret key or ephemeral secret key)
   * @param ephPkOrIvpk - The public key generated with the ephemeral secret key used in encryption
   *
   * The "odd" input stems from ivsk * ephPk == ivpk * ephSk producing the same value.
   * This is used to allow for the same decryption function to be used by both the sender and the recipient.
   *
   * @returns The decrypted log body as a buffer
   */
  protected static fromCiphertextToBuffer(
    ciphertext: Buffer | bigint[],
    ivskOrEphSk: GrumpkinScalar,
    ephPkOrIvpk: PublicKey,
  ): Buffer {
    const input = Buffer.isBuffer(ciphertext) ? ciphertext : Buffer.from(ciphertext.map((x: bigint) => Number(x)));

    const aesSecret = deriveAESSecret(ivskOrEphSk, ephPkOrIvpk);
    const key = aesSecret.subarray(0, 16);
    const iv = aesSecret.subarray(16, 32);

    const buffer = new Aes128().decryptBufferCBC(input, iv, key);
    return buffer;
  }

  /**
   * Encrypts a log body
   *
   * @param ephSk - The ephemeral secret key
   * @param ivpk - The application scoped incoming viewing key for the recipient of this log
   *
   * @returns The ciphertext of the encrypted log body
   */
  public computeCiphertext(ephSk: GrumpkinScalar, ivpk: PublicKey) {
    const aesSecret = deriveAESSecret(ephSk, ivpk);
    const key = aesSecret.subarray(0, 16);
    const iv = aesSecret.subarray(16, 32);

    const aes128 = new Aes128();
    const buffer = this.toBuffer();

    return aes128.encryptBufferCBC(buffer, iv, key);
  }
}
