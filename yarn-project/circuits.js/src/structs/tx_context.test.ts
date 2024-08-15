import { randomInt } from '@aztec/foundation/crypto';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { TX_CONTEXT_LENGTH } from '../constants.gen.js';
import { makeTxContext } from '../tests/factories.js';
import { TxContext } from './tx_context.js';

describe('TxContext', () => {
  let context: TxContext;

  beforeAll(() => {
    setupCustomSnapshotSerializers(expect);
    context = makeTxContext(randomInt(1000));
  });

  it(`serializes to buffer and deserializes it back`, () => {
    const buffer = context.toBuffer();
    const res = TxContext.fromBuffer(buffer);
    expect(res).toEqual(context);
    expect(res.isEmpty()).toBe(false);
  });

  it('number of fields matches constant', () => {
    const fields = context.toFields();
    expect(fields.length).toBe(TX_CONTEXT_LENGTH);
  });
});
