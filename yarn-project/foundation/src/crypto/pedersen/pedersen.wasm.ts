import { BarretenbergSync, Fr as FrBarretenberg } from '@aztec/bb.js';

import { Fr } from '../../fields/fields.js';
import { type Fieldable, serializeToFields } from '../../serialize/serialize.js';

/**
 * Create a pedersen commitment (point) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function pedersenCommit(input: Buffer[], offset = 0) {
  if (!input.every(i => i.length <= 32)) {
    throw new Error('All Pedersen Commit input buffers must be <= 32 bytes.');
  }
  input = input.map(i => (i.length < 32 ? Buffer.concat([Buffer.alloc(32 - i.length, 0), i]) : i));
  const point = BarretenbergSync.getSingleton().pedersenCommit(
    input.map(i => new FrBarretenberg(i)),
    offset,
  );
  // toBuffer returns Uint8Arrays (browser/worker-boundary friendly).
  // TODO: rename toTypedArray()?
  return [Buffer.from(point.x.toBuffer()), Buffer.from(point.y.toBuffer())];
}

/**
 * Create a pedersen hash (field) from an array of input fields.
 * @param input - The input fieldables to hash.
 * @param index - The separator index to use for the hash.
 * @returns The pedersen hash.
 */
export function pedersenHash(input: Fieldable[], index = 0): Fr {
  const inputFields = serializeToFields(input);
  return Fr.fromBuffer(
    Buffer.from(
      BarretenbergSync.getSingleton()
        .pedersenHash(
          inputFields.map(i => new FrBarretenberg(i.toBuffer())), // TODO(#4189): remove this stupid conversion
          index,
        )
        .toBuffer(),
    ),
  );
}

/**
 * Create a pedersen hash from an arbitrary length buffer.
 */
export function pedersenHashBuffer(input: Buffer, index = 0) {
  return Buffer.from(BarretenbergSync.getSingleton().pedersenHashBuffer(input, index).toBuffer());
}
