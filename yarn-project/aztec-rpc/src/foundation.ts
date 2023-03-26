import nodeCrypto from 'crypto';
import { Keccak } from 'sha3';

export const randomBytes = (len: number) => {
  return nodeCrypto.randomBytes(len) as Buffer;
};

export function keccak256(input: Buffer | string) {
  const inputBuf = typeof input === 'string' ? Buffer.from(input) : input;
  const hash = new Keccak(256);
  return hash.update(inputBuf).digest();
}
