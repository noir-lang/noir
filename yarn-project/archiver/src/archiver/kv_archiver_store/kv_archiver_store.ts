import {
  ContractData,
  ExtendedContractData,
  GetUnencryptedLogsResponse,
  L1ToL2Message,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  LogFilter,
  LogType,
  TxHash,
} from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore } from '@aztec/kv-store';
import { ContractClassWithId, ContractInstanceWithAddress } from '@aztec/types/contracts';

import { ArchiverDataStore, ArchiverL1SynchPoint } from '../archiver_store.js';
import { BlockStore } from './block_store.js';
import { ContractClassStore } from './contract_class_store.js';
import { ContractInstanceStore } from './contract_instance_store.js';
import { ContractStore } from './contract_store.js';
import { LogStore } from './log_store.js';
import { MessageStore } from './message_store.js';

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class KVArchiverDataStore implements ArchiverDataStore {
  #blockStore: BlockStore;
  #logStore: LogStore;
  #contractStore: ContractStore;
  #messageStore: MessageStore;
  #contractClassStore: ContractClassStore;
  #contractInstanceStore: ContractInstanceStore;

  #log = createDebugLogger('aztec:archiver:lmdb');

  constructor(db: AztecKVStore, logsMaxPageSize: number = 1000) {
    this.#blockStore = new BlockStore(db);
    this.#logStore = new LogStore(db, this.#blockStore, logsMaxPageSize);
    this.#contractStore = new ContractStore(db, this.#blockStore);
    this.#messageStore = new MessageStore(db);
    this.#contractClassStore = new ContractClassStore(db);
    this.#contractInstanceStore = new ContractInstanceStore(db);
  }

  getContractClass(id: Fr): Promise<ContractClassWithId | undefined> {
    return Promise.resolve(this.#contractClassStore.getContractClass(id));
  }

  getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return Promise.resolve(this.#contractInstanceStore.getContractInstance(address));
  }

  async addContractClasses(data: ContractClassWithId[], _blockNumber: number): Promise<boolean> {
    return (await Promise.all(data.map(c => this.#contractClassStore.addContractClass(c)))).every(Boolean);
  }

  async addContractInstances(data: ContractInstanceWithAddress[], _blockNumber: number): Promise<boolean> {
    return (await Promise.all(data.map(c => this.#contractInstanceStore.addContractInstance(c)))).every(Boolean);
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
   * The blocks returned do not contain any logs.
   *
   * @param start - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks, without any logs attached
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
   * Gets an l2 tx.
   * @param txHash - The txHash of the l2 tx.
   * @returns The requested L2 tx.
   */
  getL2Tx(txHash: TxHash): Promise<L2Tx | undefined> {
    return Promise.resolve(this.#blockStore.getL2Tx(txHash));
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
   * @param messages - The message keys to be removed from the store.
   * @param l1BlockNumber - The L1 block number for which to remove the messages.
   * @returns True if the operation is successful.
   */
  cancelPendingL1ToL2Messages(messages: Fr[], l1BlockNumber: bigint): Promise<boolean> {
    return Promise.resolve(this.#messageStore.cancelPendingMessages(messages, l1BlockNumber));
  }

  /**
   * Messages that have been published in an L2 block are confirmed.
   * Add them to the confirmed store, also remove them from the pending store.
   * @param entryKeys - The message keys to be removed from the store.
   * @param blockNumber - The block for which to add the messages.
   * @returns True if the operation is successful.
   */
  confirmL1ToL2Messages(entryKeys: Fr[]): Promise<boolean> {
    return this.#messageStore.confirmPendingMessages(entryKeys);
  }

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages, sorted by fee
   * @param limit - The number of messages to return (by default NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).
   * @returns The requested L1 to L2 message keys.
   */
  getPendingL1ToL2MessageKeys(limit: number): Promise<Fr[]> {
    return Promise.resolve(this.#messageStore.getPendingMessageKeysByFee(limit));
  }

  /**
   * Gets the confirmed L1 to L2 message corresponding to the given message key.
   * @param messageKey - The message key to look up.
   * @returns The requested L1 to L2 message or throws if not found.
   */
  getConfirmedL1ToL2Message(messageKey: Fr): Promise<L1ToL2Message> {
    try {
      return Promise.resolve(this.#messageStore.getConfirmedMessage(messageKey));
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
   * Add new extended contract data from an L2 block to the store's list.
   * @param data - List of contracts' data to be added.
   * @param blockNum - Number of the L2 block the contract data was deployed in.
   * @returns True if the operation is successful.
   */
  addExtendedContractData(data: ExtendedContractData[], blockNum: number): Promise<boolean> {
    return this.#contractStore.addExtendedContractData(data, blockNum);
  }

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    return Promise.resolve(this.#contractStore.getExtendedContractData(contractAddress));
  }

  /**
   * Lookup all extended contract data in an L2 block.
   * @param blockNumber - The block number to get all contract data from.
   * @returns All extended contract data in the block (if found).
   */
  getExtendedContractDataInBlock(blockNumber: number): Promise<ExtendedContractData[]> {
    return Promise.resolve(Array.from(this.#contractStore.getExtendedContractDataInBlock(blockNumber)));
  }

  /**
   * Get basic info for an L2 contract.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return Promise.resolve(this.#contractStore.getContractData(contractAddress));
  }

  /**
   * Get basic info for an all L2 contracts deployed in a block.
   * Contains contract address & the ethereum portal address.
   * @param blockNumber - Number of the L2 block where contracts were deployed.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  getContractDataInBlock(blockNumber: number): Promise<ContractData[]> {
    return Promise.resolve(Array.from(this.#contractStore.getContractDataInBlock(blockNumber)));
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
    const { addedMessages, cancelledMessages } = this.#messageStore.getL1BlockNumber();
    return Promise.resolve({
      addedBlock,
      addedMessages,
      cancelledMessages,
    });
  }
}
