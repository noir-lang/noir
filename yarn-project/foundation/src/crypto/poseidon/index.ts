import { BarretenbergSync, Fr } from '@aztec/bb.js';

/**
 * Create a poseidon hash (field) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function poseidonHash(input: Buffer[]): Buffer {
  return Buffer.from(
    BarretenbergSync.getSingleton()
      .poseidonHash(input.map(i => new Fr(i)))
      .toBuffer(),
  );
}
