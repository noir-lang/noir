import { randomInt } from '@aztec/foundation/crypto';

import { L2_TO_L1_MESSAGE_LENGTH } from '../constants.gen.js';
import { makeL2ToL1Message } from '../tests/factories.js';
import { L2ToL1Message } from './l2_to_l1_message.js';

describe('L2ToL1Message', () => {
  let message: L2ToL1Message;

  beforeAll(() => {
    message = makeL2ToL1Message(randomInt(1000));
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = message.toBuffer();
    const res = L2ToL1Message.fromBuffer(buffer);
    expect(res).toEqual(message);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = message.toFields();
    const res = L2ToL1Message.fromFields(fieldArray);
    expect(res).toEqual(message);
  });

  it('number of fields matches constant', () => {
    const fields = message.toFields();
    expect(fields.length).toBe(L2_TO_L1_MESSAGE_LENGTH);
  });
});
