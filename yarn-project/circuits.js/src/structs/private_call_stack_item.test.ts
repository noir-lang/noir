import { randomInt } from '@aztec/foundation/crypto';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { PRIVATE_CALL_STACK_ITEM_LENGTH } from '../constants.gen.js';
import { makePrivateCallStackItem } from '../tests/factories.js';
import { PrivateCallStackItem } from './private_call_stack_item.js';

describe('PrivateCallStackItem', () => {
  let item: PrivateCallStackItem;

  beforeAll(() => {
    setupCustomSnapshotSerializers(expect);
    item = makePrivateCallStackItem(randomInt(1000));
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
});
