import { keccak256 } from '../crypto/index.js';

/**
 * Generate a keccak256 hash of the given Ethereum message following its signed message format.
 * The function adds the Ethereum Signed Message preamble and message length before hashing the data.
 * This helps ensure that the data being signed cannot be misinterpreted as a transaction or other data.
 *
 * @param data - A Buffer containing the data to be hashed.
 * @returns A Buffer containing the keccak256 hashed Ethereum message.
 */
export function hashMessage(data: Buffer) {
  const preamble = '\x19Ethereum Signed Message:\n' + data.length;
  const preambleBuffer = Buffer.from(preamble);
  const ethMessage = Buffer.concat([preambleBuffer, data]);
  return keccak256(ethMessage);
}
