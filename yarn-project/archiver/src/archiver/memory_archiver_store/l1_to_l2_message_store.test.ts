import { InboxLeaf } from '@aztec/circuit-types';
import { INITIAL_L2_BLOCK_NUM, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { L1ToL2MessageStore } from './l1_to_l2_message_store.js';

describe('l1_to_l2_message_store', () => {
  let store: L1ToL2MessageStore;

  beforeEach(() => {
    // already adds a message to the store
    store = new L1ToL2MessageStore();
  });

  it('adds a message and correctly returns its index', () => {
    const blockNumber = 236n;
    const msgs = Array.from({ length: 10 }, (_, i) => {
      return new InboxLeaf(blockNumber, BigInt(i), Fr.random());
    });
    for (const m of msgs) {
      store.addMessage(m);
    }

    const retrievedMsgs = store.getMessages(blockNumber);
    expect(retrievedMsgs.length).toEqual(10);

    const msg = msgs[4];
    const index = store.getMessageIndex(msg.leaf);
    expect(index).toEqual(
      (blockNumber - BigInt(INITIAL_L2_BLOCK_NUM)) * BigInt(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP) + msg.index,
    );
  });
});
