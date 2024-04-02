import { type InboxLeaf } from '@aztec/circuit-types';
import {
  INITIAL_L2_BLOCK_NUM,
  L1_TO_L2_MSG_SUBTREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
} from '@aztec/circuits.js/constants';
import { type Fr } from '@aztec/foundation/fields';

/**
 * A simple in-memory implementation of an L1 to L2 message store.
 */
export class L1ToL2MessageStore {
  /**
   * A map pointing from a key in a "blockNum-messageIndex" format to the corresponding L1 to L2 message hash.
   */
  protected store: Map<string, Fr> = new Map();

  #l1ToL2MessagesSubtreeSize = 2 ** L1_TO_L2_MSG_SUBTREE_HEIGHT;

  constructor() {}

  addMessage(message: InboxLeaf) {
    if (message.index >= this.#l1ToL2MessagesSubtreeSize) {
      throw new Error(`Message index ${message.index} out of subtree range`);
    }
    const key = `${message.blockNumber}-${message.index}`;
    this.store.set(key, message.leaf);
  }

  getMessages(blockNumber: bigint): Fr[] {
    const messages: Fr[] = [];
    let undefinedMessageFound = false;
    for (let messageIndex = 0; messageIndex < this.#l1ToL2MessagesSubtreeSize; messageIndex++) {
      // This is inefficient but probably fine for now.
      const key = `${blockNumber}-${messageIndex}`;
      const message = this.store.get(key);
      if (message) {
        if (undefinedMessageFound) {
          throw new Error(`L1 to L2 message gap found in block ${blockNumber}`);
        }
        messages.push(message);
      } else {
        undefinedMessageFound = true;
        // We continue iterating over messages here to verify that there are no more messages after the undefined one.
        // --> If this was the case this would imply there is some issue with log fetching.
      }
    }
    return messages;
  }

  /**
   * Gets the first L1 to L2 message index in the L1 to L2 message tree which is greater than or equal to `startIndex`.
   * @param l1ToL2Message - The L1 to L2 message.
   * @param startIndex - The index to start searching from.
   * @returns The index of the L1 to L2 message in the L1 to L2 message tree (undefined if not found).
   */
  getMessageIndex(l1ToL2Message: Fr, startIndex: bigint): bigint | undefined {
    for (const [key, message] of this.store.entries()) {
      if (message.equals(l1ToL2Message)) {
        const keyParts = key.split('-');
        const [blockNumber, messageIndex] = [BigInt(keyParts[0]), BigInt(keyParts[1])];
        const indexInTheWholeTree =
          (blockNumber - BigInt(INITIAL_L2_BLOCK_NUM)) * BigInt(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP) + messageIndex;
        if (indexInTheWholeTree < startIndex) {
          continue;
        }
        return indexInTheWholeTree;
      }
    }
    return undefined;
  }
}
