import { makeProof } from '../tests/factories.js';
import { Proof } from './proof.js';

describe('Proof', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makeProof();
    const buffer = expected.toBuffer();
    const res = Proof.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to hex string and deserializes it back', () => {
    const expected = makeProof();
    const str = expected.toString();
    const res = Proof.fromString(str);
    expect(res).toEqual(expected);
  });
});
