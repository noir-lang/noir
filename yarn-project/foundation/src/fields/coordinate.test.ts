import { toBufferBE } from '../bigint-buffer/index.js';
import { Coordinate } from './coordinate.js';
import { Fr } from './fields.js';

const MAX_256_VALUE = 2n ** 256n - 1n;

describe('coordinate', () => {
  it('stores 256 bits in fields', () => {
    const max256Value = toBufferBE(MAX_256_VALUE, 32);
    const coordinate = Coordinate.fromBuffer(max256Value);
    // this returns a buffer containing the bit pattern split across 2 fields
    expect(coordinate.toFieldsBuffer()).toEqual(
      Buffer.concat([Buffer.alloc(1, 0), Buffer.alloc(31, 0xff), Buffer.alloc(31, 0), Buffer.alloc(1, 0xff)]),
    );
    // this returns the value in a single 32 byte buffer
    expect(coordinate.toBuffer()).toEqual(max256Value);
    // this returns the value as a big int
    expect(coordinate.toBigInt()).toBe(MAX_256_VALUE);
  });

  it('can be constructed from a field', () => {
    const field = Fr.random();
    const coordinate = Coordinate.fromField(field);
    expect(coordinate.toBuffer()).toEqual(field.toBuffer());
  });
});
