import { makeStateDiffHints } from '../../tests/factories.js';
import { StateDiffHints } from './state_diff_hints.js';

describe('StateDiffHints', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makeStateDiffHints();
    const buffer = expected.toBuffer();
    const res = StateDiffHints.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
