import { BarretenbergSync, Fr as FrBarretenberg } from '@aztec/bb.js';

import { Fr } from '../../fields/fields.js';
import { type Bufferable, serializeToBufferArray } from '../../serialize/serialize.js';

/**
 * Create a pedersen commitment (point) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function pedersenCommit(input: Buffer[]) {
  if (!input.every(i => i.length <= 32)) {
    throw new Error('All Pedersen Commit input buffers must be <= 32 bytes.');
  }
  input = input.map(i => (i.length < 32 ? Buffer.concat([Buffer.alloc(32 - i.length, 0), i]) : i));
  const point = BarretenbergSync.getSingleton().pedersenCommit(input.map(i => new FrBarretenberg(i)));
  // toBuffer returns Uint8Arrays (browser/worker-boundary friendly).
  // TODO: rename toTypedArray()?
  return [Buffer.from(point.x.toBuffer()), Buffer.from(point.y.toBuffer())];
}

/**
 * Create a pedersen hash (field) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function pedersenHash(input: Bufferable[], index = 0): Fr {
  let bufferredInput = serializeToBufferArray(input);
  if (!bufferredInput.every(i => i.length <= 32)) {
    throw new Error('All Pedersen Hash input buffers must be <= 32 bytes.');
  }
  bufferredInput = bufferredInput.map(i => (i.length < 32 ? Buffer.concat([Buffer.alloc(32 - i.length, 0), i]) : i));
  return Fr.fromBuffer(
    Buffer.from(
      BarretenbergSync.getSingleton()
        .pedersenHash(
          bufferredInput.map(i => new FrBarretenberg(i)),
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
