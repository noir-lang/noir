import { expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import { makeTxContext } from '../tests/factories.js';

describe('structs/tx', () => {
  it(`serializes and prints object`, async () => {
    const txContext = makeTxContext(1);
    await expectSerializeToMatchSnapshot(txContext.toBuffer(), 'abis__test_roundtrip_serialize_tx_context');
  });
});
