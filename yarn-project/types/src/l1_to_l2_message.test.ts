import { L1ToL2Message, L1ToL2MessageAndIndex } from './l1_to_l2_message.js';

describe('L1 to L2 message', () => {
  it('can encode an L1 to L2 message to buffer and back', () => {
    const msg = L1ToL2Message.random();
    const buffer = msg.toBuffer();
    const recovered = L1ToL2Message.fromBuffer(buffer);
    expect(recovered).toEqual(msg);
  });

  it('can encode an L1ToL2MessageAndIndex to buffer and back', () => {
    const index = BigInt(Math.floor(Math.random() * 1000)); // Generate a random BigInt
    const msg = L1ToL2Message.random();
    const l1ToL2MsgAndIndex = new L1ToL2MessageAndIndex(index, msg);

    const buffer = l1ToL2MsgAndIndex.toBuffer();
    const recovered = L1ToL2MessageAndIndex.fromBuffer(buffer);

    expect(recovered).toEqual(l1ToL2MsgAndIndex);
  });

  it('can encode an L1ToL2MessageAndIndex to string and back', () => {
    const index = BigInt(Math.floor(Math.random() * 1000)); // Generate a random BigInt
    const msg = L1ToL2Message.random();
    const l1ToL2MsgAndIndex = new L1ToL2MessageAndIndex(index, msg);

    const stringData = l1ToL2MsgAndIndex.toString();
    const recovered = L1ToL2MessageAndIndex.fromString(stringData);

    expect(recovered).toEqual(l1ToL2MsgAndIndex);
  });
});
