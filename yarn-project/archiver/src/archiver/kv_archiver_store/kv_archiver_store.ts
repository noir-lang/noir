import {
  Body,
  GetUnencryptedLogsResponse,
  L1ToL2Message,
  L2Block,
  L2BlockL2Logs,
  LogFilter,
  LogType,
  NewInboxLeaf,
  TxEffect,
  TxHash,
  TxReceipt,
} from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore } from '@aztec/kv-store';
import { ContractClassPublic, ContractInstanceWithAddress } from '@aztec/types/contracts';

import { ArchiverDataStore, ArchiverL1SynchPoint } from '../archiver_store.js';
import { BlockBodyStore } from './block_body_store.js';
import { BlockStore } from './block_store.js';
import { ContractClassStore } from './contract_class_store.js';
import { ContractInstanceStore } from './contract_instance_store.js';
import { LogStore } from './log_store.js';
import { MessageStore } from './message_store.js';

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class KVArchiverDataStore implements ArchiverDataStore {
  #blockStore: BlockStore;
  #blockBodyStore: BlockBodyStore;
  #logStore: LogStore;
  #messageStore: MessageStore;
  #contractClassStore: ContractClassStore;
  #contractInstanceStore: ContractInstanceStore;

  #log = createDebugLogger('aztec:archiver:data-store');

  constructor(db: AztecKVStore, logsMaxPageSize: number = 1000) {
    this.#blockBodyStore = new BlockBodyStore(db);
    this.#blockStore = new BlockStore(db, this.#blockBodyStore);
    this.#logStore = new LogStore(db, this.#blockStore, logsMaxPageSize);
    this.#messageStore = new MessageStore(db);
    this.#contractClassStore = new ContractClassStore(db);
    this.#contractInstanceStore = new ContractInstanceStore(db);
  }

  getContractClass(id: Fr): Promise<ContractClassPublic | undefined> {
    return Promise.resolve(this.#contractClassStore.getContractClass(id));
  }

  getContractClassIds(): Promise<Fr[]> {
    return Promise.resolve(this.#contractClassStore.getContractClassIds());
  }

  getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return Promise.resolve(this.#contractInstanceStore.getContractInstance(address));
  }

  async addContractClasses(data: ContractClassPublic[], _blockNumber: number): Promise<boolean> {
    return (await Promise.all(data.map(c => this.#contractClassStore.addContractClass(c)))).every(Boolean);
  }

  async addContractInstances(data: ContractInstanceWithAddress[], _blockNumber: number): Promise<boolean> {
    return (await Promise.all(data.map(c => this.#contractInstanceStore.addContractInstance(c)))).every(Boolean);
  }

  /**
   * Append new block bodies to the store's list.
   * @param blockBodies - The L2 block bodies to be added to the store.
   * @returns True if the operation is successful.
   */
  addBlockBodies(blockBodies: Body[]): Promise<boolean> {
    return this.#blockBodyStore.addBlockBodies(blockBodies);
  }

  /**
   * Gets block bodies that have the same txHashes as we supply.
   *
   * @param txsEffectsHashes - A list of txsEffectsHashes (body hashes).
   * @returns The requested L2 block bodies
   */
  getBlockBodies(txsEffectsHashes: Buffer[]): Promise<Body[]> {
    return this.#blockBodyStore.getBlockBodies(txsEffectsHashes);
  }

  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store.
   * @returns True if the operation is successful.
   */
  addBlocks(blocks: L2Block[]): Promise<boolean> {
    return this.#blockStore.addBlocks(blocks);
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   *
   * @param start - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks
   */
  getBlocks(start: number, limit: number): Promise<L2Block[]> {
    try {
      return Promise.resolve(Array.from(this.#blockStore.getBlocks(start, limit)));
    } catch (err) {
      // this function is sync so if any errors are thrown we need to make sure they're passed on as rejected Promises
      return Promise.reject(err);
    }
  }

  /**
   * Gets a tx effect.
   * @param txHash - The txHash of the tx corresponding to the tx effect.
   * @returns The requested tx effect (or undefined if not found).
   */
  getTxEffect(txHash: TxHash): Promise<TxEffect | undefined> {
    return Promise.resolve(this.#blockStore.getTxEffect(txHash));
  }

  /**
   * Gets a receipt of a settled tx.
   * @param txHash - The hash of a tx we try to get the receipt for.
   * @returns The requested tx receipt (or undefined if not found).
   */
  getSettledTxReceipt(txHash: TxHash): Promise<TxReceipt | undefined> {
    return Promise.resolve(this.#blockStore.getSettledTxReceipt(txHash));
  }

  /**
   * Append new logs to the store's list.
   * @param encryptedLogs - The logs to be added to the store.
   * @param unencryptedLogs - The type of the logs to be added to the store.
   * @param blockNumber - The block for which to add the logs.
   * @returns True if the operation is successful.
   */
  addLogs(
    encryptedLogs: L2BlockL2Logs | undefined,
    unencryptedLogs: L2BlockL2Logs | undefined,
    blockNumber: number,
  ): Promise<boolean> {
    return this.#logStore.addLogs(encryptedLogs, unencryptedLogs, blockNumber);
  }

  /**
   * Append new L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store.
   * @param lastMessageL1BlockNumber - The L1 block number in which the last message was emitted.
   * @returns True if the operation is successful.
   */
  addNewL1ToL2Messages(messages: NewInboxLeaf[], lastMessageL1BlockNumber: bigint): Promise<boolean> {
    return Promise.resolve(this.#messageStore.addNewL1ToL2Messages(messages, lastMessageL1BlockNumber));
  }

  /**
   * Append new pending L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store.
   * @param l1BlockNumber - The L1 block number for which to add the messages.
   * @returns True if the operation is successful.
   */
  addPendingL1ToL2Messages(messages: L1ToL2Message[], l1BlockNumber: bigint): Promise<boolean> {
    return Promise.resolve(this.#messageStore.addPendingMessages(messages, l1BlockNumber));
  }

  /**
   * Remove pending L1 to L2 messages from the store (if they were cancelled).
   * @param messages - The entry keys to be removed from the store.
   * @param l1BlockNumber - The L1 block number for which to remove the messages.
   * @returns True if the operation is successful.
   */
  cancelPendingL1ToL2EntryKeys(messages: Fr[], l1BlockNumber: bigint): Promise<boolean> {
    return Promise.resolve(this.#messageStore.cancelPendingMessages(messages, l1BlockNumber));
  }

  /**
   * Messages that have been published in an L2 block are confirmed.
   * Add them to the confirmed store, also remove them from the pending store.
   * @param entryKeys - The entry keys to be removed from the store.
   * @param blockNumber - The block for which to add the messages.
   * @returns True if the operation is successful.
   */
  confirmL1ToL2EntryKeys(entryKeys: Fr[]): Promise<boolean> {
    return this.#messageStore.confirmPendingMessages(entryKeys);
  }

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages, sorted by fee
   * @param limit - The number of messages to return (by default NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).
   * @returns The requested L1 to L2 entry keys.
   */
  getPendingL1ToL2EntryKeys(limit: number): Promise<Fr[]> {
    return Promise.resolve(this.#messageStore.getPendingEntryKeysByFee(limit));
  }

  /**
   * Gets the confirmed L1 to L2 message corresponding to the given entry key.
   * @param entryKey - The entry key to look up.
   * @returns The requested L1 to L2 message or throws if not found.
   */
  getConfirmedL1ToL2Message(entryKey: Fr): Promise<L1ToL2Message> {
    try {
      return Promise.resolve(this.#messageStore.getConfirmedMessage(entryKey));
    } catch (err) {
      return Promise.reject(err);
    }
  }

  /**
   * Gets new L1 to L2 message (to be) included in a given block.
   * @param blockNumber - L2 block number to get messages for.
   * @returns The L1 to L2 messages/leaves of the messages subtree (throws if not found).
   */
  getNewL1ToL2Messages(blockNumber: bigint): Promise<Fr[]> {
    try {
      return Promise.resolve(this.#messageStore.getNewL1ToL2Messages(blockNumber));
    } catch (err) {
      return Promise.reject(err);
    }
  }

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param start - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  getLogs(start: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]> {
    try {
      return Promise.resolve(Array.from(this.#logStore.getLogs(start, limit, logType)));
    } catch (err) {
      return Promise.reject(err);
    }
  }

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    try {
      return Promise.resolve(this.#logStore.getUnencryptedLogs(filter));
    } catch (err) {
      return Promise.reject(err);
    }
  }

  /**
   * Gets the number of the latest L2 block processed.
   * @returns The number of the latest L2 block processed.
   */
  getBlockNumber(): Promise<number> {
    return Promise.resolve(this.#blockStore.getBlockNumber());
  }

  /**
   * Gets the last L1 block number processed by the archiver
   */
  getL1BlockNumber(): Promise<ArchiverL1SynchPoint> {
    const addedBlock = this.#blockStore.getL1BlockNumber();
    const { addedMessages, cancelledMessages, newMessages } = this.#messageStore.getL1BlockNumber();
    return Promise.resolve({
      addedBlock,
      addedMessages,
      newMessages,
      cancelledMessages,
    });
  }
}
