import { TX_CONTEXT_DATA_LENGTH } from '../constants.gen.js';
import { makeTxContext } from '../tests/factories.js';
import { TxContext } from './tx_context.js';

describe('TxContext', () => {
  let context: TxContext;

  beforeAll(() => {
    const randomInt = Math.floor(Math.random() * 1000);
    context = makeTxContext(randomInt);
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

    // Value used in empty_hash test in contract_deployment_data.nr
    // console.log("hash", hash.toString());
  });
});
