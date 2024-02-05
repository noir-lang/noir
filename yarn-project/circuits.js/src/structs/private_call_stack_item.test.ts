import { PRIVATE_CALL_STACK_ITEM_LENGTH } from '../constants.gen.js';
import { makePrivateCallStackItem } from '../tests/factories.js';
import { PrivateCallStackItem } from './private_call_stack_item.js';

describe('PrivateCallStackItem', () => {
  let item: PrivateCallStackItem;

  beforeAll(() => {
    const randomInt = Math.floor(Math.random() * 1000);
    item = makePrivateCallStackItem(randomInt);
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = item.toBuffer();
    const res = PrivateCallStackItem.fromBuffer(buffer);
    expect(res).toEqual(item);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = item.toFields();
    const res = PrivateCallStackItem.fromFields(fieldArray);
    expect(res).toEqual(item);
  });

  it('number of fields matches constant', () => {
    const fields = item.toFields();
    expect(fields.length).toBe(PRIVATE_CALL_STACK_ITEM_LENGTH);
  });

  it('computes hash', () => {
    const seed = 9870243;
    const item = makePrivateCallStackItem(seed);
    const hash = item.hash();
    expect(hash).toMatchSnapshot();
  });

  it('computes empty item hash', () => {
    const item = PrivateCallStackItem.empty();
    const hash = item.hash();
    expect(hash).toMatchSnapshot();

    // Value used in empty_hash test in private_call_stack_item.nr
    // console.log("hash", hash.toString());
  });
});
