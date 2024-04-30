import { GeneratorIndex, type GrumpkinPrivateKey, type PublicKey } from '@aztec/circuits.js';
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
 * @param secretKey - The secret key used to derive shared secret.
 * @param publicKey - The public key used to derive shared secret.
 * @returns A derived AES secret key.
 * TODO(#5726): This function is called point_to_symmetric_key in Noir. I don't like that name much since point is not
 * the only input of the function. Unify naming once we have a better name.
 */
export function deriveAESSecret(secretKey: GrumpkinPrivateKey, publicKey: PublicKey): Buffer {
  const curve = new Grumpkin();
  const sharedSecret = curve.mul(publicKey, secretKey);
  const secretBuffer = Buffer.concat([sharedSecret.toBuffer(), numToUInt8(GeneratorIndex.SYMMETRIC_KEY)]);
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
 * @param ephSecretKey - The ephemeral secret key..
 * @param incomingViewingPublicKey - The note owner's incoming viewing public key.
 * @returns A Buffer containing the encrypted data and the ephemeral public key.
 */
export function encryptBuffer(
  data: Buffer,
  ephSecretKey: GrumpkinPrivateKey,
  incomingViewingPublicKey: PublicKey,
): Buffer {
  const aesSecret = deriveAESSecret(ephSecretKey, incomingViewingPublicKey);
  const aesKey = aesSecret.subarray(0, 16);
  const iv = aesSecret.subarray(16, 32);
  const cipher = createCipheriv('aes-128-cbc', aesKey, iv);
  const plaintext = Buffer.concat([iv.subarray(0, 8), data]);
  const curve = new Grumpkin();
  const ephPubKey = curve.mul(curve.generator(), ephSecretKey);

  return Buffer.concat([cipher.update(plaintext), cipher.final(), ephPubKey.toBuffer()]);
}

/**
 * Decrypts the given encrypted data buffer using the provided secret key.
 * @param data - The encrypted data buffer to be decrypted.
 * @param incomingViewingSecretKey - The secret key used for decryption.
 * @returns The decrypted plaintext as a Buffer or undefined if decryption fails.
 */
export function decryptBuffer(data: Buffer, incomingViewingSecretKey: GrumpkinPrivateKey): Buffer | undefined {
  // Extract the ephemeral public key from the end of the data
  const ephPubKey = Point.fromBuffer(data.subarray(-64));
  // Derive the AES secret key using the secret key and the ephemeral public key
  const aesSecret = deriveAESSecret(incomingViewingSecretKey, ephPubKey);
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
