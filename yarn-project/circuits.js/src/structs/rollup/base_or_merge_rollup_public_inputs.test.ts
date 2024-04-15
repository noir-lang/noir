import { makeBaseOrMergeRollupPublicInputs } from '../../tests/factories.js';
import { BaseOrMergeRollupPublicInputs } from './base_or_merge_rollup_public_inputs.js';

describe('BaseRollupPublicInputs', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = makeBaseOrMergeRollupPublicInputs();
    const buffer = expected.toBuffer();
    const res = BaseOrMergeRollupPublicInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it(`serializes to hex string and deserializes it back`, () => {
    const expected = makeBaseOrMergeRollupPublicInputs();
    const str = expected.toString();
    const res = BaseOrMergeRollupPublicInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
