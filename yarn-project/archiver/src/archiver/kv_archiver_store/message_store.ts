import { L1ToL2Message } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecCounter, AztecKVStore, AztecMap, AztecSingleton } from '@aztec/kv-store';

/**
 * A message stored in the database
 */
type Message = {
  /** The L1ToL2Message */
  message: Buffer;
  /** The message's fee */
  fee: number;
  /** Has it _ever_ been confirmed? */
  confirmed: boolean;
};

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class MessageStore {
  #messages: AztecMap<string, Message>;
  #pendingMessagesByFee: AztecCounter<[number, string]>;
  #lastL1BlockAddingMessages: AztecSingleton<bigint>;
  #lastL1BlockCancellingMessages: AztecSingleton<bigint>;

  #log = createDebugLogger('aztec:archiver:message_store');

  constructor(private db: AztecKVStore) {
    this.#messages = db.openMap('archiver_l1_to_l2_messages');
    this.#pendingMessagesByFee = db.openCounter('archiver_messages_by_fee');
    this.#lastL1BlockAddingMessages = db.openSingleton('archiver_last_l1_block_adding_messages');
    this.#lastL1BlockCancellingMessages = db.openSingleton('archiver_last_l1_block_cancelling_messages');
  }

  /**
   * Gets the last L1 block number that emitted new messages and the block that cancelled messages.
   * @returns The last L1 block number processed
   */
  getL1BlockNumber() {
    return {
      addedMessages: this.#lastL1BlockAddingMessages.get() ?? 0n,
      cancelledMessages: this.#lastL1BlockCancellingMessages.get() ?? 0n,
    };
  }

  /**
   * Append new pending L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store.
   * @param l1BlockNumber - The L1 block number for which to add the messages.
   * @returns True if the operation is successful.
   */
  addPendingMessages(messages: L1ToL2Message[], l1BlockNumber: bigint): Promise<boolean> {
    return this.db.transaction(() => {
      const lastL1BlockNumber = this.#lastL1BlockAddingMessages.get() ?? 0n;
      if (lastL1BlockNumber >= l1BlockNumber) {
        return false;
      }

      void this.#lastL1BlockAddingMessages.set(l1BlockNumber);

      for (const message of messages) {
        const messageKey = message.entryKey?.toString();
        if (!messageKey) {
          throw new Error('Message does not have an entry key');
        }

        void this.#messages.setIfNotExists(messageKey, {
          message: message.toBuffer(),
          fee: message.fee,
          confirmed: false,
        });

        void this.#pendingMessagesByFee.update([message.fee, messageKey], 1);
      }

      return true;
    });
  }

  /**
   * Remove pending L1 to L2 messages from the store (if they were cancelled).
   * @param messageKeys - The message keys to be removed from the store.
   * @param l1BlockNumber - The L1 block number for which to remove the messages.
   * @returns True if the operation is successful.
   */
  cancelPendingMessages(messageKeys: Fr[], l1BlockNumber: bigint): Promise<boolean> {
    return this.db.transaction(() => {
      const lastL1BlockNumber = this.#lastL1BlockCancellingMessages.get() ?? 0n;
      if (lastL1BlockNumber >= l1BlockNumber) {
        return false;
      }

      void this.#lastL1BlockCancellingMessages.set(l1BlockNumber);

      for (const messageKey of messageKeys) {
        const messageCtx = this.#messages.get(messageKey.toString());
        if (!messageCtx) {
          throw new Error(`Message ${messageKey.toString()} not found`);
        }

        void this.#pendingMessagesByFee.update([messageCtx.fee, messageKey.toString()], -1);
      }

      return true;
    });
  }

  /**
   * Messages that have been published in an L2 block are confirmed.
   * Add them to the confirmed store, also remove them from the pending store.
   * @param messageKeys - The message keys to be removed from the store.
   * @returns True if the operation is successful.
   */
  confirmPendingMessages(messageKeys: Fr[]): Promise<boolean> {
    return this.db.transaction(() => {
      for (const messageKey of messageKeys) {
        const messageCtx = this.#messages.get(messageKey.toString());
        if (!messageCtx) {
          throw new Error(`Message ${messageKey.toString()} not found`);
        }
        messageCtx.confirmed = true;

        void this.#messages.set(messageKey.toString(), messageCtx);
        void this.#pendingMessagesByFee.update([messageCtx.fee, messageKey.toString()], -1);
      }

      return true;
    });
  }

  /**
   * Gets the confirmed L1 to L2 message corresponding to the given message key.
   * @param messageKey - The message key to look up.
   * @returns The requested L1 to L2 message or throws if not found.
   */
  getConfirmedMessage(messageKey: Fr): L1ToL2Message {
    const messageCtx = this.#messages.get(messageKey.toString());
    if (!messageCtx) {
      throw new Error(`Message ${messageKey.toString()} not found`);
    }

    if (!messageCtx.confirmed) {
      throw new Error(`Message ${messageKey.toString()} not confirmed`);
    }

    return L1ToL2Message.fromBuffer(messageCtx.message);
  }

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages, sorted by fee
   * @param limit - The number of messages to return (by default NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).
   * @returns The requested L1 to L2 message keys.
   */
  getPendingMessageKeysByFee(limit: number): Fr[] {
    const messageKeys: Fr[] = [];

    for (const [[_, messageKey], count] of this.#pendingMessagesByFee.entries({
      reverse: true,
    })) {
      // put `count` copies of this message in the result list
      messageKeys.push(...Array(count).fill(Fr.fromString(messageKey)));
      if (messageKeys.length >= limit) {
        break;
      }
    }

    return messageKeys;
  }
}
