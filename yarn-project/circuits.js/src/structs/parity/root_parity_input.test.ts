import { makeRootParityInput } from '../../tests/factories.js';
import { RootParityInput } from './root_parity_input.js';

describe('RootParityInput', () => {
  it(`serializes a RootParityInput to buffer and deserializes it back`, () => {
    const expected = makeRootParityInput();
    const buffer = expected.toBuffer();
    const res = RootParityInput.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
