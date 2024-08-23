import { makeBlockRootRollupInputs } from '../../tests/factories.js';
import { BlockRootRollupInputs } from './block_root_rollup.js';

describe('BlockRootRollupInputs', () => {
  it(`serializes a BlockRootRollupInputs to buffer and deserializes it back`, () => {
    const expected = makeBlockRootRollupInputs();
    const buffer = expected.toBuffer();
    const res = BlockRootRollupInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it(`serializes a BlockRootRollupInputs to hex string and deserializes it back`, () => {
    const expected = makeBlockRootRollupInputs();
    const str = expected.toString();
    const res = BlockRootRollupInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
