import { Keccak } from 'sha3';

/**
 * Computes the Keccak-256 hash of the given input buffer.
 *
 * @param input - The input buffer to be hashed.
 * @returns The computed Keccak-256 hash as a Buffer.
 */
export function keccak256(input: Buffer) {
  const hash = new Keccak(256);
  return hash.update(input).digest();
}

/**
 * Computes the keccak-256 hash of a given input string and returns the result as a hexadecimal string.
 */
export function keccak256String(input: string) {
  const hash = new Keccak(256);
  hash.reset();
  hash.update(input);
  return hash.digest('hex');
}

/**
 * Computes the Keccak-224 hash of the given input buffer.
 *
 * @param input - The input buffer to be hashed.
 * @returns The computed Keccak-224 hash as a Buffer.
 */
export function keccak224(input: Buffer) {
  const hash = new Keccak(224);
  return hash.update(input).digest();
}
