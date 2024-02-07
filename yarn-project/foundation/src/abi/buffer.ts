import chunk from 'lodash.chunk';

import { Fr } from '../fields/fields.js';

/**
 * Formats a buffer as an array of fields. Splits the input into 31-byte chunks, and stores each
 * of them into a field, omitting the field's first byte, then adds zero-fields at the end until the max length.
 * @param input - Input to format.
 * @param targetLength - Length of the target array in number of fields.
 * @returns A field with the total length in bytes, followed by an array of fields such that their concatenation is equal to the input buffer, followed by enough zeroes to reach targetLength.
 */
export function bufferAsFields(input: Buffer, targetLength: number): Fr[] {
  const encoded = [
    new Fr(input.length),
    ...chunk(input, Fr.SIZE_IN_BYTES - 1).map(c => {
      const fieldBytes = Buffer.alloc(Fr.SIZE_IN_BYTES);
      Buffer.from(c).copy(fieldBytes, 1);
      return Fr.fromBuffer(fieldBytes);
    }),
  ];
  if (encoded.length > targetLength) {
    throw new Error(`Input buffer exceeds maximum size: got ${encoded.length} but max is ${targetLength}`);
  }
  // Fun fact: we cannot use padArrayEnd here since typescript cannot deal with a Tuple this big
  return [...encoded, ...Array(targetLength - encoded.length).fill(Fr.ZERO)];
}

/**
 * Recovers a buffer from an array of fields.
 * @param fields - An output from bufferAsFields.
 * @returns The recovered buffer.
 */
export function bufferFromFields(fields: Fr[]): Buffer {
  const [length, ...payload] = fields;
  return Buffer.concat(payload.map(f => f.toBuffer().subarray(1))).subarray(0, length.toNumber());
}
