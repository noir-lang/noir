import { NESTED_RECURSIVE_PROOF_LENGTH, RECURSIVE_PROOF_LENGTH } from '../../constants.gen.js';
import { makeRootParityInput } from '../../tests/factories.js';
import { RootParityInput } from './root_parity_input.js';

describe('RootParityInput', () => {
  it(`serializes a recursive proof RootParityInput to buffer and deserializes it back`, () => {
    const expected = makeRootParityInput<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH);
    const buffer = expected.toBuffer();
    const res = RootParityInput.fromBuffer(buffer, RECURSIVE_PROOF_LENGTH);
    expect(res).toEqual(expected);
  });

  it(`serializes a nested recursive proof RootParityInput to buffer and deserializes it back`, () => {
    const expected = makeRootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH);
    const buffer = expected.toBuffer();
    const res = RootParityInput.fromBuffer(buffer, NESTED_RECURSIVE_PROOF_LENGTH);
    expect(res).toEqual(expected);
  });
});
