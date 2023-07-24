import { randomBytes as cryptoRandomBytes } from 'crypto';

export function randomBytes(len: number) {
  return new Uint8Array(cryptoRandomBytes(len));
}
