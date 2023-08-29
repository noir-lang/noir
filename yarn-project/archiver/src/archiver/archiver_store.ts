import { Fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import {
  ContractData,
  ExtendedContractData,
  INITIAL_L2_BLOCK_NUM,
  L1ToL2Message,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  LogType,
  TxHash,
} from '@aztec/types';

import { L1ToL2MessageStore, PendingL1ToL2MessageStore } from './l1_to_l2_message_store.js';

/**
 * Interface describing a data store to be used by the archiver to store all its relevant data
 * (blocks, encrypted logs, aztec contract data extended contract data).
 */
export interface ArchiverDataStore {
  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store.
   * @returns True if the operation is successful.
   */
  addL2Blocks(blocks: L2Block[]): Promise<boolean>;

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks.
   */
  getL2Blocks(from: number, limit: number): Promise<L2Block[]>;

  /**
   * Gets an l2 tx.
   * @param txHash - The txHash of the l2 tx.
   * @returns The requested L2 tx.
   */
  getL2Tx(txHash: TxHash): Promise<L2Tx | undefined>;

  /**
   * Append new logs to the store's list.
   * @param data - The logs to be added to the store.
   * @param logType - The type of the logs to be added to the store.
   * @returns True if the operation is successful.
   */
  addLogs(data: L2BlockL2Logs[], logType: LogType): Promise<boolean>;

  /**
   * Append new pending L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store.
   * @returns True if the operation is successful.
   */
  addPendingL1ToL2Messages(messages: L1ToL2Message[]): Promise<boolean>;

  /**
   * Remove pending L1 to L2 messages from the store (if they were cancelled).
   * @param messageKeys - The message keys to be removed from the store.
   * @returns True if the operation is successful.
   */
  cancelPendingL1ToL2Messages(messageKeys: Fr[]): Promise<boolean>;

  /**
   * Messages that have been published in an L2 block are confirmed.
   * Add them to the confirmed store, also remove them from the pending store.
   * @param messageKeys - The message keys to be removed from the store.
   * @returns True if the operation is successful.
   */
  confirmL1ToL2Messages(messageKeys: Fr[]): Promise<boolean>;

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages, sorted by fee
   * @param limit - The number of messages to return (by default NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).
   * @returns The requested L1 to L2 message keys.
   */
  getPendingL1ToL2MessageKeys(limit: number): Promise<Fr[]>;

  /**
   * Gets the confirmed L1 to L2 message corresponding to the given message key.
   * @param messageKey - The message key to look up.
   * @returns The requested L1 to L2 message or throws if not found.
   */
  getConfirmedL1ToL2Message(messageKey: Fr): Promise<L1ToL2Message>;

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  getLogs(from: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]>;

  /**
   * Add new extended contract data from an L2 block to the store's list.
   * @param data - List of contracts' data to be added.
   * @param blockNum - Number of the L2 block the contract data was deployed in.
   * @returns True if the operation is successful.
   */
  addExtendedContractData(data: ExtendedContractData[], blockNum: number): Promise<boolean>;

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined>;

  /**
   * Lookup all extended contract data in an L2 block.
   * @param blockNum - The block number to get all contract data from.
   * @returns All extended contract data in the block (if found).
   */
  getExtendedContractDataInBlock(blockNum: number): Promise<ExtendedContractData[]>;

  /**
   * Get basic info for an L2 contract.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Get basic info for an all L2 contracts deployed in a block.
   * Contains contract address & the ethereum portal address.
   * @param l2BlockNum - Number of the L2 block where contracts were deployed.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  getContractDataInBlock(l2BlockNum: number): Promise<ContractData[] | undefined>;

  /**
   * Gets the number of the latest L2 block processed.
   * @returns The number of the latest L2 block processed.
   */
  getBlockNumber(): Promise<number>;

  /**
   * Gets the length of L2 blocks in store.
   * @returns The length of L2 Blocks stored.
   */
  getBlocksLength(): number;
}

/**
 * Simple, in-memory implementation of an archiver data store.
 */
export class MemoryArchiverStore implements ArchiverDataStore {
  /**
   * An array containing all the L2 blocks that have been fetched so far.
   */
  private l2Blocks: L2Block[] = [];

  /**
   * An array containing all the L2 Txs in the L2 blocks that have been fetched so far.
   */
  private l2Txs: L2Tx[] = [];

  /**
   * An array containing all the encrypted logs that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private encryptedLogs: L2BlockL2Logs[] = [];

  /**
   * An array containing all the unencrypted logs that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private unencryptedLogs: L2BlockL2Logs[] = [];

  /**
   * A sparse array containing all the extended contract data that have been fetched so far.
   */
  private extendedContractDataByBlock: (ExtendedContractData[] | undefined)[] = [];

  /**
   * A mapping of contract address to extended contract data.
   */
  private extendedContractData: Map<string, ExtendedContractData> = new Map();

  /**
   * Contains all the confirmed L1 to L2 messages (i.e. messages that were consumed in an L2 block)
   * It is a map of entryKey to the corresponding L1 to L2 message and the number of times it has appeared
   */
  private confirmedL1ToL2Messages: L1ToL2MessageStore = new L1ToL2MessageStore();

  /**
   * Contains all the pending L1 to L2 messages (accounts for duplication of messages)
   */
  private pendingL1ToL2Messages: PendingL1ToL2MessageStore = new PendingL1ToL2MessageStore();

  constructor() {}

  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store.
   * @returns True if the operation is successful (always in this implementation).
   */
  public addL2Blocks(blocks: L2Block[]): Promise<boolean> {
    this.l2Blocks.push(...blocks);
    this.l2Txs.push(...blocks.flatMap(b => b.getTxs()));
    return Promise.resolve(true);
  }

  /**
   * Append new logs to the store's list.
   * @param data - The logs to be added to the store.
   * @param logType - The type of the logs to be added to the store.
   * @returns True if the operation is successful.
   */
  addLogs(data: L2BlockL2Logs[], logType: LogType): Promise<boolean> {
    logType === LogType.ENCRYPTED ? this.encryptedLogs.push(...data) : this.unencryptedLogs.push(...data);
    return Promise.resolve(true);
  }

  /**
   * Append new pending L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store.
   * @returns True if the operation is successful (always in this implementation).
   */
  public addPendingL1ToL2Messages(messages: L1ToL2Message[]): Promise<boolean> {
    for (const msg of messages) {
      this.pendingL1ToL2Messages.addMessage(msg.entryKey!, msg);
    }
    return Promise.resolve(true);
  }

  /**
   * Remove pending L1 to L2 messages from the store (if they were cancelled).
   * @param messageKeys - The message keys to be removed from the store.
   * @returns True if the operation is successful (always in this implementation).
   */
  public cancelPendingL1ToL2Messages(messageKeys: Fr[]): Promise<boolean> {
    messageKeys.forEach(messageKey => {
      this.pendingL1ToL2Messages.removeMessage(messageKey);
    });
    return Promise.resolve(true);
  }

  /**
   * Messages that have been published in an L2 block are confirmed.
   * Add them to the confirmed store, also remove them from the pending store.
   * @param messageKeys - The message keys to be removed from the store.
   * @returns True if the operation is successful (always in this implementation).
   */
  public confirmL1ToL2Messages(messageKeys: Fr[]): Promise<boolean> {
    messageKeys.forEach(messageKey => {
      this.confirmedL1ToL2Messages.addMessage(messageKey, this.pendingL1ToL2Messages.getMessage(messageKey)!);
      this.pendingL1ToL2Messages.removeMessage(messageKey);
    });
    return Promise.resolve(true);
  }

  /**
   * Store new extended contract data from an L2 block to the store's list.
   * @param data - List of contracts' data to be added.
   * @param blockNum - Number of the L2 block the contract data was deployed in.
   * @returns True if the operation is successful (always in this implementation).
   */
  public addExtendedContractData(data: ExtendedContractData[], blockNum: number): Promise<boolean> {
    // Add to the contracts mapping
    for (const contractData of data) {
      const key = contractData.contractData.contractAddress.toString();
      this.extendedContractData.set(key, contractData);
    }

    // Add the index per block
    if (this.extendedContractDataByBlock[blockNum]?.length) {
      this.extendedContractDataByBlock[blockNum]?.push(...data);
    } else {
      this.extendedContractDataByBlock[blockNum] = [...data];
    }
    return Promise.resolve(true);
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks.
   */
  public getL2Blocks(from: number, limit: number): Promise<L2Block[]> {
    // Return an empty array if we are outside of range
    if (limit < 1) {
      throw new Error(`Invalid block range from: ${from}, limit: ${limit}`);
    }
    if (from < INITIAL_L2_BLOCK_NUM || from > this.l2Blocks.length) {
      return Promise.resolve([]);
    }
    const startIndex = from - INITIAL_L2_BLOCK_NUM;
    const endIndex = startIndex + limit;
    return Promise.resolve(this.l2Blocks.slice(startIndex, endIndex));
  }

  /**
   * Gets an l2 tx.
   * @param txHash - The txHash of the l2 tx.
   * @returns The requested L2 tx.
   */
  public getL2Tx(txHash: TxHash): Promise<L2Tx | undefined> {
    const l2Tx = this.l2Txs.find(tx => tx.txHash.equals(txHash));
    return Promise.resolve(l2Tx);
  }

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages, sorted by fee
   * @param limit - The number of messages to return (by default NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).
   * @returns The requested L1 to L2 message keys.
   */
  public getPendingL1ToL2MessageKeys(limit: number = NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP): Promise<Fr[]> {
    return Promise.resolve(this.pendingL1ToL2Messages.getMessageKeys(limit));
  }

  /**
   * Gets the confirmed L1 to L2 message corresponding to the given message key.
   * @param messageKey - The message key to look up.
   * @returns The requested L1 to L2 message or throws if not found.
   */
  public getConfirmedL1ToL2Message(messageKey: Fr): Promise<L1ToL2Message> {
    const message = this.confirmedL1ToL2Messages.getMessage(messageKey);
    if (!message) {
      throw new Error(`L1 to L2 Message with key ${messageKey.toString()} not found in the confirmed messages store`);
    }
    return Promise.resolve(message);
  }

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  getLogs(from: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]> {
    if (from < INITIAL_L2_BLOCK_NUM || limit < 1) {
      throw new Error(`Invalid block range from: ${from}, limit: ${limit}`);
    }
    const logs = logType === LogType.ENCRYPTED ? this.encryptedLogs : this.unencryptedLogs;
    if (from > logs.length) {
      return Promise.resolve([]);
    }
    const startIndex = from - INITIAL_L2_BLOCK_NUM;
    const endIndex = startIndex + limit;
    return Promise.resolve(logs.slice(startIndex, endIndex));
  }

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    const result = this.extendedContractData.get(contractAddress.toString());
    return Promise.resolve(result);
  }

  /**
   * Lookup all contract data in an L2 block.
   * @param blockNum - The block number to get all contract data from.
   * @returns All extended contract data in the block (if found).
   */
  public getExtendedContractDataInBlock(blockNum: number): Promise<ExtendedContractData[]> {
    if (blockNum > this.l2Blocks.length) {
      return Promise.resolve([]);
    }
    return Promise.resolve(this.extendedContractDataByBlock[blockNum] || []);
  }

  /**
   * Get basic info for an L2 contract.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    if (contractAddress.isZero()) {
      return Promise.resolve(undefined);
    }
    for (const block of this.l2Blocks) {
      for (const contractData of block.newContractData) {
        if (contractData.contractAddress.equals(contractAddress)) {
          return Promise.resolve(contractData);
        }
      }
    }
    return Promise.resolve(undefined);
  }

  /**
   * Get basic info for an all L2 contracts deployed in a block.
   * Contains contract address & the ethereum portal address.
   * @param l2BlockNum - Number of the L2 block where contracts were deployed.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getContractDataInBlock(l2BlockNum: number): Promise<ContractData[] | undefined> {
    if (l2BlockNum > this.l2Blocks.length) {
      return Promise.resolve([]);
    }
    const block = this.l2Blocks[l2BlockNum];
    return Promise.resolve(block.newContractData);
  }

  /**
   * Gets the number of the latest L2 block processed.
   * @returns The number of the latest L2 block processed.
   */
  public getBlockNumber(): Promise<number> {
    if (this.l2Blocks.length === 0) return Promise.resolve(INITIAL_L2_BLOCK_NUM - 1);
    return Promise.resolve(this.l2Blocks[this.l2Blocks.length - 1].number);
  }

  /**
   * Gets the length of L2 blocks in store.
   * @returns The length of L2 Blocks array.
   */
  public getBlocksLength(): number {
    return this.l2Blocks.length;
  }
}
