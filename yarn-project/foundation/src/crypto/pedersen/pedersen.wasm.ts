import { BarretenbergSync, Fr } from '@aztec/bb.js';

// Get the singleton. This constructs (if not already) the barretenberg sync api within bb.js itself.
// This can be called from multiple other modules as needed, and it ensures it's only constructed once.
const api = await BarretenbergSync.getSingleton();

/**
 * Create a pedersen commitment (point) from an array of input fields.
 * Left pads any inputs less than 32 bytes.
 */
export function pedersenCommit(input: Buffer[]) {
  if (!input.every(i => i.length <= 32)) {
    throw new Error('All input buffers must be <= 32 bytes.');
  }
  input = input.map(i => (i.length < 32 ? Buffer.concat([Buffer.alloc(32 - i.length, 0), i]) : i));
  const point = api.pedersenCommit(input.map(i => new Fr(i)));
  // toBuffer returns Uint8Arrays (browser/worker-boundary friendly).
  // TODO: rename toTypedArray()?
  return [Buffer.from(point.x.toBuffer()), Buffer.from(point.y.toBuffer())];
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
  return Buffer.from(
    api
      .pedersenHash(
        input.map(i => new Fr(i)),
        index,
      )
      .toBuffer(),
  );
}

/**
 * Create a pedersen hash from an arbitrary length buffer.
 */
export function pedersenHashBuffer(input: Buffer, index = 0) {
  return Buffer.from(api.pedersenHashBuffer(input, index).toBuffer());
}
