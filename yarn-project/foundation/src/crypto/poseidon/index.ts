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
export function poseidon2Hash(input: Fieldable[], _index = 0): Fr {
  const inputFields = serializeToFields(input);
  return Fr.fromBuffer(
    Buffer.from(
      BarretenbergSync.getSingleton()
        .poseidon2Hash(
          inputFields.map(i => new FrBarretenberg(i.toBuffer())), // TODO(#4189): remove this stupid conversion
          // index, // TODO: enable once the barretenberg API supports it
        )
        .toBuffer(),
    ),
  );
}

/**
 * Runs a Poseidon2 permutation.
 * @param input the input state. Expected to be of size 4.
 * @returns the output state, size 4.
 */
export function poseidon2Permutation(input: Fieldable[]): Fr[] {
  const inputFields = serializeToFields(input);
  // We'd like this assertion but it's not possible to use it in the browser.
  // assert(input.length === 4, 'Input state must be of size 4');
  const res = BarretenbergSync.getSingleton().poseidon2Permutation(
    inputFields.map(i => new FrBarretenberg(i.toBuffer())),
  );
  // We'd like this assertion but it's not possible to use it in the browser.
  // assert(res.length === 4, 'Output state must be of size 4');
  return res.map(o => Fr.fromBuffer(Buffer.from(o.toBuffer())));
}
