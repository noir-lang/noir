import { Keccak } from 'sha3';

export function keccak(input: Buffer) {
  const hash = new Keccak(256);
  return hash.update(input).digest();
}
