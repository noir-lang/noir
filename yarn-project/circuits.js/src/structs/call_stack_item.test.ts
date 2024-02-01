import { makePrivateCallStackItem } from '../tests/factories.js';
import { PrivateCallStackItem } from './call_stack_item.js';

describe('PrivateCallStackItem', () => {
  it('serializes to buffer and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makePrivateCallStackItem(randomInt);
    const buffer = expected.toBuffer();
    const res = PrivateCallStackItem.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to field array and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makePrivateCallStackItem(randomInt);

    const fieldArray = expected.toFields();
    const res = PrivateCallStackItem.fromFields(fieldArray);
    expect(res).toEqual(expected);
  });

  it('computes hash', () => {
    const seed = 9870243;
    const PrivateCallStackItem = makePrivateCallStackItem(seed);
    const hash = PrivateCallStackItem.hash();
    expect(hash).toMatchSnapshot();
  });
});
