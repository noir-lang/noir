import nodeCrypto from 'crypto';
import isNode from 'detect-node';

import { RandomnessSingleton } from './randomness_singleton.js';

// limit of Crypto.getRandomValues()
// https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
const MAX_BYTES = 65536;

const getWebCrypto = () => {
  if (typeof window !== 'undefined' && window.crypto) {
    return window.crypto;
  }
  if (typeof self !== 'undefined' && self.crypto) {
    return self.crypto;
  }
  return undefined;
};

export const randomBytes = (len: number) => {
  const singleton = RandomnessSingleton.getInstance();

  if (singleton.isDeterministic()) {
    return singleton.getBytes(len);
  }

  if (isNode) {
    return nodeCrypto.randomBytes(len) as Buffer;
  }

  const crypto = getWebCrypto();
  if (!crypto) {
    throw new Error('randomBytes UnsupportedEnvironment');
  }

  const buf = Buffer.allocUnsafe(len);
  if (len > MAX_BYTES) {
    // this is the max bytes crypto.getRandomValues
    // can do at once see https://developer.mozilla.org/en-US/docs/Web/API/window.crypto.getRandomValues
    for (let generated = 0; generated < len; generated += MAX_BYTES) {
      // buffer.slice automatically checks if the end is past the end of
      // the buffer so we don't have to here
      crypto.getRandomValues(buf.slice(generated, generated + MAX_BYTES));
    }
  } else {
    crypto.getRandomValues(buf);
  }

  return buf;
};

/**
 * Generate a random integer less than max.
 * @param max - The maximum value.
 * @returns A random integer.
 *
 * TODO(#3949): This is insecure as it's modulo biased. Nuke or safeguard before mainnet.
 */
export const randomInt = (max: number) => {
  const randomBuffer = randomBytes(6); // Generate a buffer of 6 random bytes.
  const randomInt = parseInt(randomBuffer.toString('hex'), 16); // Convert buffer to a large integer.
  return randomInt % max; // Use modulo to ensure the result is less than max.
};

/**
 * Generate a random bigint less than max.
 * @param max - The maximum value.
 * @returns A random bigint.
 *
 * TODO(#3949): This is insecure as it's modulo biased. Nuke or safeguard before mainnet.
 */
export const randomBigInt = (max: bigint) => {
  const randomBuffer = randomBytes(8); // Generate a buffer of 8 random bytes.
  const randomBigInt = BigInt(`0x${randomBuffer.toString('hex')}`); // Convert buffer to a large integer.
  return randomBigInt % max; // Use modulo to ensure the result is less than max.
};

/**
 * Generate a random boolean value.
 * @returns A random boolean value.
 */
export const randomBoolean = () => {
  const randomByte = randomBytes(1)[0]; // Generate a single random byte.
  return randomByte % 2 === 0; // Use modulo to determine if the byte is even or odd.
};
