import { BarretenbergSync, Fr as FrBarretenberg } from '@aztec/bb.js';

import { Fr } from '../../fields/fields.js';

/**
 * Create a poseidon hash (field) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function poseidonHash(input: Buffer[]): Fr {
  return Fr.fromBuffer(
    Buffer.from(
      BarretenbergSync.getSingleton()
        .poseidonHash(input.map(i => new FrBarretenberg(i)))
        .toBuffer(),
    ),
  );
}
