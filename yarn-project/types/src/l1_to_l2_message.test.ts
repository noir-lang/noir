import { L1ToL2Message } from './l1_to_l2_message.js';

describe('L1 to L2 message', () => {
  it('can encode an L1 to L2 message to buffer and back', () => {
    const msg = L1ToL2Message.random();
    const buffer = msg.toBuffer();
    const recovered = L1ToL2Message.fromBuffer(buffer);
    expect(recovered).toEqual(msg);
  });
});
