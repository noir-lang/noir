import { makePreviousRollupBlockData } from '../../tests/factories.js';
import { PreviousRollupBlockData } from './previous_rollup_block_data.js';

describe('PreviousRollupBlockData', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makePreviousRollupBlockData();
    const buffer = expected.toBuffer();
    const res = PreviousRollupBlockData.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
