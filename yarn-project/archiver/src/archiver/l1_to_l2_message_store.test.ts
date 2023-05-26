import { Fr } from '@aztec/foundation/fields';
import { L1ToL2MessageStore } from './l1_to_l2_message_store.js';
import { L1Actor, L1ToL2Message, L2Actor } from '@aztec/types';

describe('l1_to_l2_message_store', () => {
  let store: L1ToL2MessageStore;
  let entryKey: Fr;
  let msg: L1ToL2Message;

  beforeEach(() => {
    // already adds a message to the store
    store = new L1ToL2MessageStore();
    entryKey = Fr.random();
    msg = L1ToL2Message.random();
  });

  it('addMessage adds a message', () => {
    store.addMessage(entryKey, msg);
    expect(store.getMessage(entryKey)).toEqual(msg);
  });

  it('addMessage increments the count if the message is already in the store', () => {
    store.addMessage(entryKey, msg);
    store.addMessage(entryKey, msg);
    expect(store.getMessageAndCount(entryKey)).toEqual({ message: msg, count: 2 });
  });

  it('removeMessage removes the message if the count is 1', () => {
    store.addMessage(entryKey, msg);
    store.removeMessage(entryKey);
    expect(store.getMessage(entryKey)).toBeUndefined();
  });

  it("handles case when removing a message that doesn't exist", () => {
    expect(() => store.removeMessage(new Fr(0))).not.toThrow();
  });

  it('removeMessage decrements the count if the message is already in the store', () => {
    store.addMessage(entryKey, msg);
    store.addMessage(entryKey, msg);
    store.addMessage(entryKey, msg);
    store.removeMessage(entryKey);
    expect(store.getMessageAndCount(entryKey)).toEqual({ message: msg, count: 2 });
  });

  it('get messages for an empty store', () => {
    const store = new L1ToL2MessageStore();
    expect(store.getMessages(10)).toEqual([]);
  });

  it('getMessages returns an empty array if take is 0', () => {
    store.addMessage(entryKey, msg);
    expect(store.getMessages(0)).toEqual([]);
  });

  it('get messages for a non-empty store when take > number of messages in store', () => {
    const store = new L1ToL2MessageStore();
    const entryKeys = [1, 2, 3, 4, 5].map(x => new Fr(x));
    entryKeys.forEach(entryKey => {
      store.addMessage(entryKey, L1ToL2Message.random());
    });
    expect(store.getMessages(10).length).toEqual(5);
  });

  it('get messages returns messages sorted by fees and also includes multiple of the same message', () => {
    const store = new L1ToL2MessageStore();
    const entryKeys = [1, 2, 3, 3, 3, 4].map(x => new Fr(x));
    entryKeys.forEach(entryKey => {
      // set msg.fee to entryKey to test the sort.
      const msg = new L1ToL2Message(
        L1Actor.random(),
        L2Actor.random(),
        Fr.random(),
        Fr.random(),
        100,
        Number(entryKey.value),
        entryKey,
      );
      store.addMessage(entryKey, msg);
    });
    const expectedMessgeFees = [4, 3, 3, 3]; // the top 4.
    const receivedMessageFees = store.getMessages(4).map(msg => msg.fee);
    expect(receivedMessageFees).toEqual(expectedMessgeFees);
  });
});
