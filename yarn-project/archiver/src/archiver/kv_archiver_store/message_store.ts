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
        const entryKey = message.entryKey?.toString();
        if (!entryKey) {
          throw new Error('Message does not have an entry key');
        }

        void this.#messages.setIfNotExists(entryKey, {
          message: message.toBuffer(),
          fee: message.fee,
          confirmed: false,
        });

        void this.#pendingMessagesByFee.update([message.fee, entryKey], 1);
      }

      return true;
    });
  }

  /**
   * Remove pending L1 to L2 messages from the store (if they were cancelled).
   * @param entryKeys - The entry keys to be removed from the store.
   * @param l1BlockNumber - The L1 block number for which to remove the messages.
   * @returns True if the operation is successful.
   */
  cancelPendingMessages(entryKeys: Fr[], l1BlockNumber: bigint): Promise<boolean> {
    return this.db.transaction(() => {
      const lastL1BlockNumber = this.#lastL1BlockCancellingMessages.get() ?? 0n;
      if (lastL1BlockNumber >= l1BlockNumber) {
        return false;
      }

      void this.#lastL1BlockCancellingMessages.set(l1BlockNumber);

      for (const entryKey of entryKeys) {
        const messageCtx = this.#messages.get(entryKey.toString());
        if (!messageCtx) {
          throw new Error(`Message ${entryKey.toString()} not found`);
        }

        void this.#pendingMessagesByFee.update([messageCtx.fee, entryKey.toString()], -1);
      }

      return true;
    });
  }

  /**
   * Messages that have been published in an L2 block are confirmed.
   * Add them to the confirmed store, also remove them from the pending store.
   * @param entryKeys - The entry keys to be removed from the store.
   * @returns True if the operation is successful.
   */
  confirmPendingMessages(entryKeys: Fr[]): Promise<boolean> {
    return this.db.transaction(() => {
      for (const entryKey of entryKeys) {
        if (entryKey.equals(Fr.ZERO)) {
          continue;
        }

        const messageCtx = this.#messages.get(entryKey.toString());
        if (!messageCtx) {
          throw new Error(`Message ${entryKey.toString()} not found`);
        }
        messageCtx.confirmed = true;

        void this.#messages.set(entryKey.toString(), messageCtx);
        void this.#pendingMessagesByFee.update([messageCtx.fee, entryKey.toString()], -1);
      }

      return true;
    });
  }

  /**
   * Gets the confirmed L1 to L2 message corresponding to the given entry key.
   * @param entryKey - The entry key to look up.
   * @returns The requested L1 to L2 message or throws if not found.
   */
  getConfirmedMessage(entryKey: Fr): L1ToL2Message {
    const messageCtx = this.#messages.get(entryKey.toString());
    if (!messageCtx) {
      throw new Error(`Message ${entryKey.toString()} not found`);
    }

    if (!messageCtx.confirmed) {
      throw new Error(`Message ${entryKey.toString()} not confirmed`);
    }

    return L1ToL2Message.fromBuffer(messageCtx.message);
  }

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages, sorted by fee
   * @param limit - The number of messages to return (by default NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).
   * @returns The requested L1 to L2 entry keys.
   */
  getPendingEntryKeysByFee(limit: number): Fr[] {
    const entryKeys: Fr[] = [];

    for (const [[_, entryKey], count] of this.#pendingMessagesByFee.entries({
      reverse: true,
    })) {
      // put `count` copies of this message in the result list
      entryKeys.push(...Array(count).fill(Fr.fromString(entryKey)));
      if (entryKeys.length >= limit) {
        break;
      }
    }

    return entryKeys;
  }
}
