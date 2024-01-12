import { GrumpkinPrivateKey, PublicKey } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { sha256 } from '@aztec/foundation/crypto';
import { Point } from '@aztec/foundation/fields';
import { numToUInt8 } from '@aztec/foundation/serialize';

import { createCipheriv, createDecipheriv } from 'browserify-cipher';

/**
 * Derive an AES secret key using Elliptic Curve Diffie-Hellman (ECDH) and SHA-256.
 * The function takes in an ECDH public key, a private key, and a Grumpkin instance to compute
 * the shared secret. The shared secret is then hashed using SHA-256 to produce the final
 * AES secret key.
 *
 * @param ecdhPubKey - The ECDH public key represented as a PublicKey object.
 * @param ecdhPrivKey - The ECDH private key represented as a Buffer object.
 * @param grumpkin - The curve to use for curve operations.
 * @returns A Buffer containing the derived AES secret key.
 */
export function deriveAESSecret(ecdhPubKey: PublicKey, ecdhPrivKey: GrumpkinPrivateKey, curve: Grumpkin): Buffer {
  const sharedSecret = curve.mul(ecdhPubKey, ecdhPrivKey);
  const secretBuffer = Buffer.concat([sharedSecret.toBuffer(), numToUInt8(1)]);
  const hash = sha256(secretBuffer);
  return hash;
}

/**
 * Encrypt a given data buffer using the owner's public key and an ephemeral private key.
 * The encrypted data includes the original data, AES secret derived from ECDH shared secret,
 * and the ephemeral public key. The encryption is done using the 'aes-128-cbc' algorithm
 * with the provided curve instance for elliptic curve operations.
 *
 * @param data - The data buffer to be encrypted.
 * @param ownerPubKey - The owner's public key as a PublicKey instance.
 * @param ephPrivKey - The ephemeral private key as a Buffer instance.
 * @param curve - The curve instance used for elliptic curve operations.
 * @returns A Buffer containing the encrypted data and the ephemeral public key.
 */
export function encryptBuffer(
  data: Buffer,
  ownerPubKey: PublicKey,
  ephPrivKey: GrumpkinPrivateKey,
  curve: Grumpkin,
): Buffer {
  const aesSecret = deriveAESSecret(ownerPubKey, ephPrivKey, curve);
  const aesKey = aesSecret.subarray(0, 16);
  const iv = aesSecret.subarray(16, 32);
  const cipher = createCipheriv('aes-128-cbc', aesKey, iv);
  const plaintext = Buffer.concat([iv.subarray(0, 8), data]);
  const ephPubKey = curve.mul(curve.generator(), ephPrivKey);
  return Buffer.concat([cipher.update(plaintext), cipher.final(), ephPubKey.toBuffer()]);
}

/**
 * Decrypts the given encrypted data buffer using the owner's private key and a Grumpkin curve.
 * Extracts the ephemeral public key from the input data, derives the AES secret using
 * the owner's private key, and decrypts the plaintext.
 * If the decryption is successful, returns the decrypted plaintext, otherwise returns undefined.
 *
 * @param data - The encrypted data buffer to be decrypted.
 * @param ownerPrivKey - The private key of the owner used for decryption.
 * @param curve - The curve object used in the decryption process.
 * @returns The decrypted plaintext as a Buffer or undefined if decryption fails.
 */
export function decryptBuffer(data: Buffer, ownerPrivKey: GrumpkinPrivateKey, curve: Grumpkin): Buffer | undefined {
  const ephPubKey = Point.fromBuffer(data.subarray(-64));
  const aesSecret = deriveAESSecret(ephPubKey, ownerPrivKey, curve);
  const aesKey = aesSecret.subarray(0, 16);
  const iv = aesSecret.subarray(16, 32);
  const cipher = createDecipheriv('aes-128-cbc', aesKey, iv);
  try {
    const plaintext = Buffer.concat([cipher.update(data.subarray(0, -64)), cipher.final()]);
    if (plaintext.subarray(0, 8).equals(iv.subarray(0, 8))) {
      return plaintext.subarray(8);
    }
  } catch (e) {
    return;
  }
}
