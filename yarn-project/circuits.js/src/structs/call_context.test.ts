import { randomInt } from '@aztec/foundation/crypto';

import { CALL_CONTEXT_LENGTH } from '../constants.gen.js';
import { makeCallContext } from '../tests/factories.js';
import { CallContext } from './call_context.js';

describe('CallContext', () => {
  let callContext: CallContext;

  beforeAll(() => {
    callContext = makeCallContext(randomInt(1000));
  });

  it(`serializes to buffer and deserializes it back`, () => {
    const buffer = callContext.toBuffer();
    const res = CallContext.fromBuffer(buffer);
    expect(res).toEqual(callContext);
    expect(res.isEmpty()).toBe(false);
  });

  it('number of fields matches constant', () => {
    const fields = callContext.toFields();
    expect(fields.length).toBe(CALL_CONTEXT_LENGTH);
  });
});
