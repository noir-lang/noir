import { makeRootParityInputs } from '../../tests/factories.js';
import { RootParityInputs } from './root_parity_inputs.js';

describe('RootParityInputs', () => {
  it(`serializes a RootParityInputs to buffer and deserializes it back`, () => {
    const expected = makeRootParityInputs();
    const buffer = expected.toBuffer();
    const res = RootParityInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it(`serializes a RootParityInputs to hex string and deserializes it back`, () => {
    const expected = makeRootParityInputs();
    const str = expected.toString();
    const res = RootParityInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
