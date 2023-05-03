import { createCipheriv, createDecipheriv } from 'browserify-cipher';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { numToUInt8 } from '@aztec/foundation/serialize';
import { sha256 } from '@aztec/foundation/crypto';
import { Point } from '@aztec/foundation/fields';

export function deriveAESSecret(ecdhPubKey: Point, ecdhPrivKey: Buffer, grumpkin: Grumpkin): Buffer {
  const sharedSecret = grumpkin.mul(ecdhPubKey.toBuffer(), ecdhPrivKey);
  const secretBuffer = Buffer.concat([sharedSecret, numToUInt8(1)]);
  const hash = sha256(secretBuffer);
  return hash;
}

export function encryptBuffer(data: Buffer, ownerPubKey: Point, ephPrivKey: Buffer, grumpkin: Grumpkin): Buffer {
  const aesSecret = deriveAESSecret(ownerPubKey, ephPrivKey, grumpkin);
  const aesKey = aesSecret.subarray(0, 16);
  const iv = aesSecret.subarray(16, 32);
  const cipher = createCipheriv('aes-128-cbc', aesKey, iv);
  const plaintext = Buffer.concat([iv.subarray(0, 8), data]);
  const ephPubKey = grumpkin.mul(Grumpkin.generator, ephPrivKey);
  return Buffer.concat([cipher.update(plaintext), cipher.final(), ephPubKey]);
}

export function decryptBuffer(data: Buffer, ownerPrivKey: Buffer, grumpkin: Grumpkin): Buffer | undefined {
  const ephPubKey = Point.fromBuffer(data.subarray(-64));
  const aesSecret = deriveAESSecret(ephPubKey, ownerPrivKey, grumpkin);
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
