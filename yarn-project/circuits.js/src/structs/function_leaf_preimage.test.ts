import { FunctionSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';

import { FUNCTION_LEAF_PREIMAGE_LENGTH } from '../constants.gen.js';
import { FunctionLeafPreimage } from './function_leaf_preimage.js';

describe('FunctionLeafPreimage', () => {
  let leaf: FunctionLeafPreimage;

  beforeAll(() => {
    leaf = new FunctionLeafPreimage(new FunctionSelector(8972), false, true, Fr.ZERO, Fr.ZERO);
  });

  it(`serializes to buffer and deserializes it back`, () => {
    const buffer = leaf.toBuffer();
    const res = FunctionLeafPreimage.fromBuffer(buffer);
    expect(res).toEqual(leaf);
  });

  it('number of fields matches constant', () => {
    const fields = leaf.toFields();
    expect(fields.length).toBe(FUNCTION_LEAF_PREIMAGE_LENGTH);
  });

  it('computes a function leaf', () => {
    const emptyLeaf = new FunctionLeafPreimage(new FunctionSelector(0), false, false, Fr.ZERO, Fr.ZERO);
    const hash = emptyLeaf.hash();
    expect(hash).toMatchSnapshot();

    // Value used in empty_hash test in function_leaf_preimage.nr
    // console.log("hash", hash.toString());
  });
});
