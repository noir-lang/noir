import { Keccak } from 'sha3';

export function keccak(input: Buffer) {
  const hash = new Keccak(256);
  return hash.update(input).digest();
}

export function keccak256String(input: string) {
  const hash = new Keccak(256);
  hash.reset();
  hash.update(input);
  return hash.digest('hex');
}
