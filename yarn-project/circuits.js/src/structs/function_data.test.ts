import { FunctionSelector } from '@aztec/foundation/abi';

import { FunctionData } from './function_data.js';

describe('FunctionData', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = new FunctionData(new FunctionSelector(123), false, true, true);
    const buffer = expected.toBuffer();
    const res = FunctionData.fromBuffer(buffer);
    expect(res).toEqual(expected);
    expect(res.isEmpty()).toBe(false);
  });
});
