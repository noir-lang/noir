import { makeMergeRollupInputs } from '../../tests/factories.js';
import { MergeRollupInputs } from './merge_rollup.js';

describe('MergeRollupInputs', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makeMergeRollupInputs();
    const buffer = expected.toBuffer();
    const res = MergeRollupInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to hex string and deserializes it back', () => {
    const expected = makeMergeRollupInputs();
    const str = expected.toString();
    const res = MergeRollupInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
