import { randomInt } from '@aztec/foundation/crypto';

import { makePublicCallStackItem } from '../tests/factories.js';
import { PublicCallStackItem } from './public_call_stack_item.js';

describe('PublicCallStackItem', () => {
  it('serializes to buffer and deserializes it back', () => {
    const expected = makePublicCallStackItem(randomInt(1000));
    const buffer = expected.toBuffer();
    const res = PublicCallStackItem.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });
});
