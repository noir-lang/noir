import { makeBaseRollupInputs } from '../../tests/factories.js';
import { BaseRollupInputs } from './base_rollup.js';

describe('BaseRollupInputs', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makeBaseRollupInputs();
    const buffer = expected.toBuffer();
    const res = BaseRollupInputs.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to hex string and deserializes it back', () => {
    const expected = makeBaseRollupInputs();
    const str = expected.toString();
    const res = BaseRollupInputs.fromString(str);
    expect(res).toEqual(expected);
  });
});
