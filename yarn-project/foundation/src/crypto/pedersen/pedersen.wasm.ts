import { Pedersen } from '@aztec/bb.js';

const pedersen = await Pedersen.new();

/**
 * Create a pedersen commitment (point) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function pedersenCommit(input: Buffer[]) {
  if (!input.every(i => i.length <= 32)) {
    throw new Error('All input buffers must be <= 32 bytes.');
  }
  input = input.map(i => (i.length < 32 ? Buffer.concat([Buffer.alloc(32 - i.length, 0), i]) : i));
  const [x, y] = pedersen.pedersenCommit(input);
  return [Buffer.from(x), Buffer.from(y)];
}

/**
 * Create a pedersen hash (field) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function pedersenHash(input: Buffer[], index = 0) {
  if (!input.every(i => i.length <= 32)) {
    throw new Error('All input buffers must be <= 32 bytes.');
  }
  input = input.map(i => (i.length < 32 ? Buffer.concat([Buffer.alloc(32 - i.length, 0), i]) : i));
  return Buffer.from(pedersen.pedersenHash(input, index));
}
