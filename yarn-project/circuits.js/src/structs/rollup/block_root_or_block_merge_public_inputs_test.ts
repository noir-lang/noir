import { makeBlockRootOrBlockMergeRollupPublicInputs } from '../../tests/factories.js';
import { BlockRootOrBlockMergePublicInputs } from './block_root_or_block_merge_public_inputs.js';

describe('BlockRootOrBlockMergePublicInputs', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = makeBlockRootOrBlockMergeRollupPublicInputs();
    const buffer = expected.toBuffer();
    const res = BlockRootOrBlockMergePublicInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it(`serializes to hex string and deserializes it back`, () => {
    const expected = makeBlockRootOrBlockMergeRollupPublicInputs();
    const str = expected.toString();
    const res = BlockRootOrBlockMergePublicInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
