import { makeRootRollupPublicInputs } from '../../tests/factories.js';
import { RootRollupPublicInputs } from './root_rollup.js';

describe('structs/root_rollup', () => {
  it(`serializes a RootRollupPublicInputs to buffer and deserializes it back`, () => {
    const expected = makeRootRollupPublicInputs();
    const buffer = expected.toBuffer();
    const res = RootRollupPublicInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it(`serializes a RootRollupPublicInputs to hex string and deserializes it back`, () => {
    const expected = makeRootRollupPublicInputs();
    const str = expected.toString();
    const res = RootRollupPublicInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
