import {
  ContractData,
  ExtendedContractData,
  ExtendedUnencryptedL2Log,
  GetUnencryptedLogsResponse,
  INITIAL_L2_BLOCK_NUM,
  L1ToL2Message,
  L2Block,
  L2BlockContext,
  L2BlockL2Logs,
  L2Tx,
  LogFilter,
  LogId,
  LogType,
  TxHash,
  UnencryptedL2Log,
} from '@aztec/circuit-types';
import { Fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { ContractClassWithId, ContractInstanceWithAddress } from '@aztec/types/contracts';

import { ArchiverDataStore } from '../archiver_store.js';
import { L1ToL2MessageStore, PendingL1ToL2MessageStore } from './l1_to_l2_message_store.js';

/**
 * Simple, in-memory implementation of an archiver data store.
 */
export class MemoryArchiverStore implements ArchiverDataStore {
  /**
   * An array containing all the L2 blocks that have been fetched so far.
   */
  private l2BlockContexts: L2BlockContext[] = [];

  /**
   * An array containing all the L2 Txs in the L2 blocks that have been fetched so far.
   */
  private l2Txs: L2Tx[] = [];

  /**
   * An array containing all the encrypted logs that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private encryptedLogsPerBlock: L2BlockL2Logs[] = [];

  /**
   * An array containing all the unencrypted logs that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private unencryptedLogsPerBlock: L2BlockL2Logs[] = [];

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

  private contractClasses: Map<string, ContractClassWithId> = new Map();

  private contractInstances: Map<string, ContractInstanceWithAddress> = new Map();

  private lastL1BlockAddedMessages: bigint = 0n;
  private lastL1BlockCancelledMessages: bigint = 0n;

  constructor(
    /** The max number of logs that can be obtained in 1 "getUnencryptedLogs" call. */
    public readonly maxLogs: number,
  ) {}

  public getContractClass(id: Fr): Promise<ContractClassWithId | undefined> {
    return Promise.resolve(this.contractClasses.get(id.toString()));
  }

  public getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return Promise.resolve(this.contractInstances.get(address.toString()));
  }

  public addContractClasses(data: ContractClassWithId[], _blockNumber: number): Promise<boolean> {
    for (const contractClass of data) {
      this.contractClasses.set(contractClass.id.toString(), contractClass);
    }
    return Promise.resolve(true);
  }

  public addContractInstances(data: ContractInstanceWithAddress[], _blockNumber: number): Promise<boolean> {
    for (const contractInstance of data) {
      this.contractInstances.set(contractInstance.address.toString(), contractInstance);
    }
    return Promise.resolve(true);
  }

  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store.
   * @returns True if the operation is successful (always in this implementation).
   */
  public addBlocks(blocks: L2Block[]): Promise<boolean> {
    this.l2BlockContexts.push(...blocks.map(block => new L2BlockContext(block)));
    this.l2Txs.push(...blocks.flatMap(b => b.getTxs()));
    return Promise.resolve(true);
  }

  /**
   * Append new logs to the store's list.
   * @param encryptedLogs - The encrypted logs to be added to the store.
   * @param unencryptedLogs - The unencrypted logs to be added to the store.
   * @param blockNumber - The block for which to add the logs.
   * @returns True if the operation is successful.
   */
  addLogs(encryptedLogs: L2BlockL2Logs, unencryptedLogs: L2BlockL2Logs, blockNumber: number): Promise<boolean> {
    if (encryptedLogs) {
      this.encryptedLogsPerBlock[blockNumber - INITIAL_L2_BLOCK_NUM] = encryptedLogs;
    }

    if (unencryptedLogs) {
      this.unencryptedLogsPerBlock[blockNumber - INITIAL_L2_BLOCK_NUM] = unencryptedLogs;
    }

    return Promise.resolve(true);
  }

  /**
   * Append new pending L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store.
   * @param l1BlockNumber - The L1 block number for which to add the messages.
   * @returns True if the operation is successful (always in this implementation).
   */
  public addPendingL1ToL2Messages(messages: L1ToL2Message[], l1BlockNumber: bigint): Promise<boolean> {
    if (l1BlockNumber <= this.lastL1BlockAddedMessages) {
      return Promise.resolve(false);
    }

    this.lastL1BlockAddedMessages = l1BlockNumber;
    for (const message of messages) {
      this.pendingL1ToL2Messages.addMessage(message.entryKey!, message);
    }
    return Promise.resolve(true);
  }

  /**
   * Remove pending L1 to L2 messages from the store (if they were cancelled).
   * @param messages - The message keys to be removed from the store.
   * @param l1BlockNumber - The L1 block number for which to remove the messages.
   * @returns True if the operation is successful (always in this implementation).
   */
  public cancelPendingL1ToL2Messages(messages: Fr[], l1BlockNumber: bigint): Promise<boolean> {
    if (l1BlockNumber <= this.lastL1BlockCancelledMessages) {
      return Promise.resolve(false);
    }

    this.lastL1BlockCancelledMessages = l1BlockNumber;
    messages.forEach(entryKey => {
      this.pendingL1ToL2Messages.removeMessage(entryKey);
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
   * @remarks When "from" is smaller than genesis block number, blocks from the beginning are returned.
   */
  public getBlocks(from: number, limit: number): Promise<L2Block[]> {
    // Return an empty array if we are outside of range
    if (limit < 1) {
      return Promise.reject(new Error(`Invalid limit: ${limit}`));
    }

    const fromIndex = Math.max(from - INITIAL_L2_BLOCK_NUM, 0);
    if (fromIndex >= this.l2BlockContexts.length) {
      return Promise.resolve([]);
    }

    const toIndex = fromIndex + limit;
    return Promise.resolve(this.l2BlockContexts.slice(fromIndex, toIndex).map(blockContext => blockContext.block));
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
      throw new Error(`Invalid limit: ${limit}`);
    }
    const logs = logType === LogType.ENCRYPTED ? this.encryptedLogsPerBlock : this.unencryptedLogsPerBlock;
    if (from > logs.length) {
      return Promise.resolve([]);
    }
    const startIndex = from - INITIAL_L2_BLOCK_NUM;
    const endIndex = startIndex + limit;
    return Promise.resolve(logs.slice(startIndex, endIndex));
  }

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   * @remarks Works by doing an intersection of all params in the filter.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    let txHash: TxHash | undefined;
    let fromBlockIndex = 0;
    let toBlockIndex = this.unencryptedLogsPerBlock.length;
    let txIndexInBlock = 0;
    let logIndexInTx = 0;

    if (filter.afterLog) {
      // Continuation parameter is set --> tx hash is ignored
      if (filter.fromBlock == undefined || filter.fromBlock <= filter.afterLog.blockNumber) {
        fromBlockIndex = filter.afterLog.blockNumber - INITIAL_L2_BLOCK_NUM;
        txIndexInBlock = filter.afterLog.txIndex;
        logIndexInTx = filter.afterLog.logIndex + 1; // We want to start from the next log
      } else {
        fromBlockIndex = filter.fromBlock - INITIAL_L2_BLOCK_NUM;
      }
    } else {
      txHash = filter.txHash;

      if (filter.fromBlock !== undefined) {
        fromBlockIndex = filter.fromBlock - INITIAL_L2_BLOCK_NUM;
      }
    }

    if (filter.toBlock !== undefined) {
      toBlockIndex = filter.toBlock - INITIAL_L2_BLOCK_NUM;
    }

    // Ensure the indices are within block array bounds
    fromBlockIndex = Math.max(fromBlockIndex, 0);
    toBlockIndex = Math.min(toBlockIndex, this.unencryptedLogsPerBlock.length);

    if (fromBlockIndex > this.unencryptedLogsPerBlock.length || toBlockIndex < fromBlockIndex || toBlockIndex <= 0) {
      return Promise.resolve({
        logs: [],
        maxLogsHit: false,
      });
    }

    const contractAddress = filter.contractAddress;
    const selector = filter.selector;

    const logs: ExtendedUnencryptedL2Log[] = [];

    for (; fromBlockIndex < toBlockIndex; fromBlockIndex++) {
      const blockContext = this.l2BlockContexts[fromBlockIndex];
      const blockLogs = this.unencryptedLogsPerBlock[fromBlockIndex];
      for (; txIndexInBlock < blockLogs.txLogs.length; txIndexInBlock++) {
        const txLogs = blockLogs.txLogs[txIndexInBlock].unrollLogs().map(log => UnencryptedL2Log.fromBuffer(log));
        for (; logIndexInTx < txLogs.length; logIndexInTx++) {
          const log = txLogs[logIndexInTx];
          if (
            (!txHash || blockContext.getTxHash(txIndexInBlock).equals(txHash)) &&
            (!contractAddress || log.contractAddress.equals(contractAddress)) &&
            (!selector || log.selector.equals(selector))
          ) {
            logs.push(
              new ExtendedUnencryptedL2Log(new LogId(blockContext.block.number, txIndexInBlock, logIndexInTx), log),
            );
            if (logs.length === this.maxLogs) {
              return Promise.resolve({
                logs,
                maxLogsHit: true,
              });
            }
          }
        }
        logIndexInTx = 0;
      }
      txIndexInBlock = 0;
    }

    return Promise.resolve({
      logs,
      maxLogsHit: false,
    });
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
    if (blockNum > this.l2BlockContexts.length) {
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
    for (const blockContext of this.l2BlockContexts) {
      for (const contractData of blockContext.block.newContractData) {
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
    if (l2BlockNum > this.l2BlockContexts.length) {
      return Promise.resolve([]);
    }
    const block: L2Block | undefined = this.l2BlockContexts[l2BlockNum - INITIAL_L2_BLOCK_NUM]?.block;
    return Promise.resolve(block?.newContractData);
  }

  /**
   * Gets the number of the latest L2 block processed.
   * @returns The number of the latest L2 block processed.
   */
  public getBlockNumber(): Promise<number> {
    if (this.l2BlockContexts.length === 0) {
      return Promise.resolve(INITIAL_L2_BLOCK_NUM - 1);
    }
    return Promise.resolve(this.l2BlockContexts[this.l2BlockContexts.length - 1].block.number);
  }

  public getL1BlockNumber() {
    const addedBlock = this.l2BlockContexts[this.l2BlockContexts.length - 1]?.block?.getL1BlockNumber() ?? 0n;
    const addedMessages = this.lastL1BlockAddedMessages;
    const cancelledMessages = this.lastL1BlockCancelledMessages;

    return Promise.resolve({
      addedBlock,
      addedMessages,
      cancelledMessages,
    });
  }
}
