import { Keccak } from 'sha3';

/**
 * Generate a Keccak-256 hash of the given input Buffer.
 * The function takes a Buffer as an input and returns a Buffer containing the hash result.
 * It is commonly used for hashing data in Ethereum and other blockchain applications.
 *
 * @param input - The input data to be hashed, provided as a Buffer.
 * @returns A Buffer representing the Keccak-256 hash of the input data.
 */
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

/**
 * Generates a sha3 hash by applying the keccak256 hashing algorithm to a given input string.
 * The resulting hash is then prefixed with '0x' to indicate that it's a hex-encoded value.
 * This function is commonly used for Ethereum address hashing and smart contract function signatures.
 *
 * @param input - The input string to be hashed using the keccak256 algorithm.
 * @returns A hex-encoded sha3 hash string, prefixed with '0x'.
 */
export function sha3(input: string) {
  return '0x' + keccak256String(input);
}
