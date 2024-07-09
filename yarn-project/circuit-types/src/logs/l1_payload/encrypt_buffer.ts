import { type GrumpkinScalar, type PublicKey } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { Point } from '@aztec/foundation/fields';

import { createCipheriv, createDecipheriv } from 'browserify-cipher';

import { deriveAESSecret } from './encryption_utils.js';

/**
 * Encrypt a given data buffer using the owner's public key and an ephemeral private key.
 * The encrypted data includes the original data, AES secret derived from ECDH shared secret,
 * and the ephemeral public key. The encryption is done using the 'aes-128-cbc' algorithm
 * with the provided curve instance for elliptic curve operations.
 *
 * @param data - The data buffer to be encrypted.
 * @param ephSecretKey - The ephemeral secret key..
 * @param incomingViewingPublicKey - The note owner's incoming viewing public key.
 * @returns A Buffer containing the encrypted data and the ephemeral public key.
 */
export function encryptBuffer(data: Buffer, ephSecretKey: GrumpkinScalar, incomingViewingPublicKey: PublicKey): Buffer {
  const aesSecret = deriveAESSecret(ephSecretKey, incomingViewingPublicKey);
  const aesKey = aesSecret.subarray(0, 16);
  const iv = aesSecret.subarray(16, 32);
  const cipher = createCipheriv('aes-128-cbc', aesKey, iv);
  const plaintext = Buffer.concat([iv.subarray(0, 8), data]);
  const curve = new Grumpkin();
  const ephPubKey = curve.mul(curve.generator(), ephSecretKey);

  // We encrypt eth pub key without the isInfinite flag because infinite point is not a valid pub key
  return Buffer.concat([cipher.update(plaintext), cipher.final(), ephPubKey.toBuffer()]);
}

/**
 * Decrypts the given encrypted data buffer using the provided secret key.
 * @param data - The encrypted data buffer to be decrypted.
 * @param incomingViewingSecretKey - The secret key used for decryption.
 * @returns The decrypted plaintext as a Buffer or undefined if decryption fails.
 */
export function decryptBuffer(data: Buffer, incomingViewingSecretKey: GrumpkinScalar): Buffer | undefined {
  // Extract the ephemeral public key from the end of the data
  const ephPubKey = Point.fromBuffer(data.subarray(-Point.SIZE_IN_BYTES));
  // Derive the AES secret key using the secret key and the ephemeral public key
  const aesSecret = deriveAESSecret(incomingViewingSecretKey, ephPubKey);
  const aesKey = aesSecret.subarray(0, 16);
  const iv = aesSecret.subarray(16, 32);
  const cipher = createDecipheriv('aes-128-cbc', aesKey, iv);
  try {
    const plaintext = Buffer.concat([cipher.update(data.subarray(0, -Point.SIZE_IN_BYTES)), cipher.final()]);
    if (plaintext.subarray(0, 8).equals(iv.subarray(0, 8))) {
      return plaintext.subarray(8);
    }
  } catch (e) {
    return;
  }
}
