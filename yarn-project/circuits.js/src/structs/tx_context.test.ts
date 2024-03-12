import { randomInt } from '@aztec/foundation/crypto';
import { setupCustomSnapshotSerializers, updateInlineTestData } from '@aztec/foundation/testing';

import { TX_CONTEXT_DATA_LENGTH } from '../constants.gen.js';
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
    expect(fields.length).toBe(TX_CONTEXT_DATA_LENGTH);
  });

  it('computes empty hash', () => {
    const tc = TxContext.empty();
    expect(tc.isEmpty()).toBe(true);

    const hash = tc.hash();
    expect(hash).toMatchSnapshot();

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/noir-protocol-circuits/crates/types/src/transaction/tx_context.nr',
      'test_data_empty_hash',
      hash.toString(),
    );
  });
});
