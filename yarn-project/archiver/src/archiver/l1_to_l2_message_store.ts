import { Fr } from '@aztec/foundation/fields';
import { L1ToL2Message } from '@aztec/types';

/**
 * A simple in-memory implementation of an L1 to L2 message store
 * that handles message duplication.
 */
export class L1ToL2MessageStore {
  /**
   * A map containing the message key to the corresponding L1 to L2
   * messages (and the number of times the message has been seen).
   */
  protected store: Map<bigint, L1ToL2MessageAndCount> = new Map();

  constructor() {}

  addMessage(messageKey: Fr, msg: L1ToL2Message) {
    const messageKeyBigInt = messageKey.value;
    const msgAndCount = this.store.get(messageKeyBigInt);
    if (msgAndCount) {
      msgAndCount.count++;
    } else {
      this.store.set(messageKeyBigInt, { message: msg, count: 1 });
    }
  }

  getMessage(messageKey: Fr): L1ToL2Message | undefined {
    return this.store.get(messageKey.value)?.message;
  }

  getMessageAndCount(messageKey: Fr): L1ToL2MessageAndCount | undefined {
    return this.store.get(messageKey.value);
  }
}

/**
 * Specifically for the store that will hold pending messages
 * for removing messages or fetching multiple messages.
 */
export class PendingL1ToL2MessageStore extends L1ToL2MessageStore {
  getMessageKeys(limit: number): Fr[] {
    if (limit < 1) {
      return [];
    }
    // fetch `limit` number of messages from the store with the highest fee.
    // Note the store has multiple of the same message. So if a message has count 2, include both of them in the result:
    const messages: Fr[] = [];
    const sortedMessages = Array.from(this.store.values()).sort((a, b) => b.message.fee - a.message.fee);
    for (const messageAndCount of sortedMessages) {
      for (let i = 0; i < messageAndCount.count; i++) {
        messages.push(messageAndCount.message.entryKey!);
        if (messages.length === limit) {
          return messages;
        }
      }
    }
    return messages;
  }

  removeMessage(messageKey: Fr) {
    // ignore 0 - messageKey is a hash, so a 0 can probabilistically never occur. It is best to skip it.
    if (messageKey.equals(Fr.ZERO)) return;
    const messageKeyBigInt = messageKey.value;
    const msgAndCount = this.store.get(messageKeyBigInt);
    if (!msgAndCount) {
      throw new Error(`Unable to remove message: L1 to L2 Message with key ${messageKeyBigInt} not found in store`);
    }
    if (msgAndCount.count > 1) {
      msgAndCount.count--;
    } else {
      this.store.delete(messageKeyBigInt);
    }
  }
}

/**
 * Useful to keep track of the number of times a message has been seen.
 */
type L1ToL2MessageAndCount = {
  /**
   * The message.
   */
  message: L1ToL2Message;
  /**
   * The number of times the message has been seen.
   */
  count: number;
};
