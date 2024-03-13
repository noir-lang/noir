import { makeParityPublicInputs } from '../../tests/factories.js';
import { ParityPublicInputs } from './parity_public_inputs.js';

describe('ParityPublicInputs', () => {
  it(`serializes a ParityPublicInputs to buffer and deserializes it back`, () => {
    const expected = makeParityPublicInputs();
    const buffer = expected.toBuffer();
    const res = ParityPublicInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
