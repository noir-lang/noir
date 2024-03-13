import { makeBaseParityInputs } from '../../tests/factories.js';
import { BaseParityInputs } from './base_parity_inputs.js';

describe('BaseParityInputs', () => {
  it(`serializes a BaseParityInputs to buffer and deserializes it back`, () => {
    const expected = makeBaseParityInputs();
    const buffer = expected.toBuffer();
    const res = BaseParityInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
