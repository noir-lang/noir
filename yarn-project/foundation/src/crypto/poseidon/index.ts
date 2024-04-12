import { BarretenbergSync, Fr as FrBarretenberg } from '@aztec/bb.js';

import { Fr } from '../../fields/fields.js';
import { type Fieldable, serializeToFields } from '../../serialize/serialize.js';

/**
 * Create a poseidon hash (field) from an array of input fields.
 * @param input - The input fields to hash.
 * @param index - The separator index to use for the hash.
 * @returns The poseidon hash.
 * TODO(#5714): enable index once barretenberg API supports it
 */
export function poseidonHash(input: Fieldable[], _index = 0): Fr {
  const inputFields = serializeToFields(input);
  return Fr.fromBuffer(
    Buffer.from(
      BarretenbergSync.getSingleton()
        .poseidonHash(
          inputFields.map(i => new FrBarretenberg(i.toBuffer())), // TODO(#4189): remove this stupid conversion
          // index, // TODO: enable once the barretenberg API supports it
        )
        .toBuffer(),
    ),
  );
}
