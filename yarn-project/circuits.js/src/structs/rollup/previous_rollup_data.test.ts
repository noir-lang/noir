import { makePreviousRollupData } from '../../tests/factories.js';
import { PreviousRollupData } from './previous_rollup_data.js';

describe('PreviousRollupData', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makePreviousRollupData();
    const buffer = expected.toBuffer();
    const res = PreviousRollupData.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
