import { makePublicCallStackItem } from '../tests/factories.js';
import { PublicCallStackItem } from './public_call_stack_item.js';

describe('PublicCallStackItem', () => {
  it('serializes to buffer and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makePublicCallStackItem(randomInt);
    const buffer = expected.toBuffer();
    const res = PublicCallStackItem.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to field array and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makePublicCallStackItem(randomInt);

    const fieldArray = expected.toFields();
    const res = PublicCallStackItem.fromFields(fieldArray);
    expect(res).toEqual(expected);
  });

  it('computes hash', () => {
    const seed = 9870243;
    const item = makePublicCallStackItem(seed);
    const hash = item.hash();
    expect(hash).toMatchSnapshot();
  });
});
