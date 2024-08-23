import { makeBlockMergeRollupInputs } from '../../tests/factories.js';
import { BlockMergeRollupInputs } from './block_merge_rollup.js';

describe('BlockMergeRollupInputs', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makeBlockMergeRollupInputs();
    const buffer = expected.toBuffer();
    const res = BlockMergeRollupInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to hex string and deserializes it back', () => {
    const expected = makeBlockMergeRollupInputs();
    const str = expected.toString();
    const res = BlockMergeRollupInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
