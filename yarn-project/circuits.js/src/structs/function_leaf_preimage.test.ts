import { FunctionSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';

import { FunctionLeafPreimage } from './function_leaf_preimage.js';

describe('FunctionLeafPreimage', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = new FunctionLeafPreimage(new FunctionSelector(8972), false, true, Fr.ZERO, Fr.ZERO);
    const buffer = expected.toBuffer();
    const res = FunctionLeafPreimage.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
