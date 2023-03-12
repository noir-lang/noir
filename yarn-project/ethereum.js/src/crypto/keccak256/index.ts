import { Keccak } from 'sha3';

export function keccak256(input: Buffer) {
  const hash = new Keccak(256);
  return hash.update(input).digest();
}

/**
 * Takes a string hex input e.g. `deadbeef` and returns the same.
 */
export function keccak256String(input: string) {
  const hash = new Keccak(256);
  hash.reset();
  hash.update(input);
  return hash.digest('hex');
}

export function sha3(input: string) {
  return '0x' + keccak256String(input);
}
