import { Fr } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { createDebugLogger } from '@aztec/foundation/log';
import {
  CancelledL1ToL2Message,
  ContractData,
  ExtendedContractData,
  ExtendedUnencryptedL2Log,
  GetUnencryptedLogsResponse,
  INITIAL_L2_BLOCK_NUM,
  L1ToL2Message,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  LogFilter,
  LogId,
  LogType,
  PendingL1ToL2Message,
  TxHash,
  UnencryptedL2Log,
} from '@aztec/types';

import { Database, RangeOptions, RootDatabase } from 'lmdb';

import { ArchiverDataStore } from './archiver_store.js';

/* eslint-disable */
type L1ToL2MessageAndCount = {
  message: Buffer;
  pendingCount: number;
  confirmedCount: number;
};

type L1ToL2MessageBlockKey = `${string}:${'newMessage' | 'cancelledMessage'}:${number}`;

function l1ToL2MessageBlockKey(
  l1BlockNumber: bigint,
  key: 'newMessage' | 'cancelledMessage',
  indexInBlock: number,
): L1ToL2MessageBlockKey {
  return `${toBufferBE(l1BlockNumber, 32).toString('hex')}:${key}:${indexInBlock}`;
}

type BlockIndexValue = [blockNumber: number, index: number];

type BlockContext = {
  block?: Uint8Array;
  l1BlockNumber?: Uint8Array;
  encryptedLogs?: Uint8Array;
  unencryptedLogs?: Uint8Array;
  extendedContractData?: Array<Uint8Array>;
};
/* eslint-enable */

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class LMDBArchiverStore implements ArchiverDataStore {
  #tables: {
    /** Where block information will be stored */
    blocks: Database<BlockContext, number>;
    /** Transactions index */
    txIndex: Database<BlockIndexValue, Buffer>;
    /** Contracts index */
    contractIndex: Database<BlockIndexValue, Buffer>;
    /** L1 to L2 messages */
    l1ToL2Messages: Database<L1ToL2MessageAndCount, Buffer>;
    /** Which blocks emitted which messages */
    l1ToL2MessagesByBlock: Database<Buffer, L1ToL2MessageBlockKey>;
    /** Pending L1 to L2 messages sorted by their fee, in buckets (dupSort=true)  */
    pendingMessagesByFee: Database<Buffer, number>;
  };

  #logsMaxPageSize: number;

  #log = createDebugLogger('aztec:archiver:lmdb');

  constructor(db: RootDatabase, logsMaxPageSize: number = 1000) {
    this.#tables = {
      blocks: db.openDB('blocks', {
        keyEncoding: 'uint32',
        encoding: 'msgpack',
      }),
      txIndex: db.openDB('tx_index', {
        keyEncoding: 'binary',
        encoding: 'msgpack',
      }),
      contractIndex: db.openDB('contract_index', {
        keyEncoding: 'binary',
        encoding: 'msgpack',
      }),
      l1ToL2Messages: db.openDB('l1_to_l2_messages', {
        keyEncoding: 'binary',
        encoding: 'msgpack',
      }),
      l1ToL2MessagesByBlock: db.openDB('l1_to_l2_message_nonces', {
        keyEncoding: 'ordered-binary',
        encoding: 'binary',
      }),
      pendingMessagesByFee: db.openDB('pending_messages_by_fee', {
        keyEncoding: 'ordered-binary',
        encoding: 'binary',
        dupSort: true,
      }),
    };

    this.#logsMaxPageSize = logsMaxPageSize;
  }

  public async close() {
    await Promise.all(Object.values(this.#tables).map(table => table.close()));
  }

  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store.
   * @returns True if the operation is successful.
   */
  addBlocks(blocks: L2Block[]): Promise<boolean> {
    // LMDB transactions are shared across databases, so we can use a single transaction for all the writes
    // https://github.com/kriszyp/lmdb-js/blob/67505a979ab63187953355a88747a7ad703d50b6/README.md#dbopendbdatabase-stringnamestring
    return this.#tables.blocks.transaction(() => {
      for (const block of blocks) {
        const blockCtx = this.#tables.blocks.get(block.number) ?? {};
        blockCtx.block = block.toBuffer();
        blockCtx.l1BlockNumber = toBufferBE(block.getL1BlockNumber(), 32);

        // no need to await, all writes are enqueued in the transaction
        // awaiting would interrupt the execution flow of this callback and "leak" the transaction to some other part
        // of the system and any writes would then be part of our transaction here
        void this.#tables.blocks.put(block.number, blockCtx);

        for (const [i, tx] of block.getTxs().entries()) {
          if (tx.txHash.isZero()) {
            continue;
          }
          void this.#tables.txIndex.put(tx.txHash.buffer, [block.number, i]);
        }

        for (const [i, contractData] of block.newContractData.entries()) {
          if (contractData.contractAddress.isZero()) {
            continue;
          }

          void this.#tables.contractIndex.put(contractData.contractAddress.toBuffer(), [block.number, i]);
        }
      }

      return true;
    });
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param start - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks.
   */
  getBlocks(start: number, limit: number): Promise<L2Block[]> {
    try {
      const blocks = this.#tables.blocks
        .getRange(this.#computeBlockRange(start, limit))
        .filter(({ value }) => value.block)
        .map(({ value }) => {
          const block = L2Block.fromBuffer(asBuffer(value.block!));
          if (value.encryptedLogs) {
            block.attachLogs(L2BlockL2Logs.fromBuffer(asBuffer(value.encryptedLogs)), LogType.ENCRYPTED);
          }

          if (value.unencryptedLogs) {
            block.attachLogs(L2BlockL2Logs.fromBuffer(asBuffer(value.unencryptedLogs)), LogType.UNENCRYPTED);
          }

          return block;
        }).asArray;

      return Promise.resolve(blocks);
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
    const [blockNumber, txIndex] = this.#tables.txIndex.get(txHash.buffer) ?? [];
    if (typeof blockNumber !== 'number' || typeof txIndex !== 'number') {
      return Promise.resolve(undefined);
    }

    const block = this.#getBlock(blockNumber, true);
    return Promise.resolve(block?.getTx(txIndex));
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
    return this.#tables.blocks.transaction(() => {
      const blockCtx = this.#tables.blocks.get(blockNumber) ?? {};

      if (encryptedLogs) {
        blockCtx.encryptedLogs = encryptedLogs.toBuffer();
      }

      if (unencryptedLogs) {
        blockCtx.unencryptedLogs = unencryptedLogs.toBuffer();
      }

      void this.#tables.blocks.put(blockNumber, blockCtx);
      return true;
    });
  }

  /**
   * Append new pending L1 to L2 messages to the store.
   * @param messages - The L1 to L2 messages to be added to the store.
   * @returns True if the operation is successful.
   */
  addPendingL1ToL2Messages(messages: PendingL1ToL2Message[]): Promise<boolean> {
    return this.#tables.l1ToL2Messages.transaction(() => {
      for (const { message, blockNumber, indexInBlock } of messages) {
        const messageKey = message.entryKey?.toBuffer();
        if (!messageKey) {
          throw new Error('Message does not have an entry key');
        }

        const dupeKey = l1ToL2MessageBlockKey(blockNumber, 'newMessage', indexInBlock);
        const messageInBlock = this.#tables.l1ToL2MessagesByBlock.get(dupeKey);

        if (messageInBlock?.equals(messageKey)) {
          continue;
        } else {
          if (messageInBlock) {
            this.#log(
              `Previously add pending message ${messageInBlock.toString(
                'hex',
              )} at ${dupeKey.toString()}, now got ${messageKey.toString('hex')}`,
            );
          }

          void this.#tables.l1ToL2MessagesByBlock.put(dupeKey, messageKey);
        }

        let messageWithCount = this.#tables.l1ToL2Messages.get(messageKey);
        if (!messageWithCount) {
          messageWithCount = {
            message: message.toBuffer(),
            pendingCount: 0,
            confirmedCount: 0,
          };
          void this.#tables.l1ToL2Messages.put(messageKey, messageWithCount);
        }

        this.#updateMessageCountInTx(messageKey, message, 1, 0);
      }
      return true;
    });
  }

  /**
   * Remove pending L1 to L2 messages from the store (if they were cancelled).
   * @param messages - The message keys to be removed from the store.
   * @returns True if the operation is successful.
   */
  cancelPendingL1ToL2Messages(messages: CancelledL1ToL2Message[]): Promise<boolean> {
    return this.#tables.l1ToL2Messages.transaction(() => {
      for (const { blockNumber, indexInBlock, entryKey } of messages) {
        const messageKey = entryKey.toBuffer();
        const dupeKey = l1ToL2MessageBlockKey(blockNumber, 'cancelledMessage', indexInBlock);
        const messageInBlock = this.#tables.l1ToL2MessagesByBlock.get(dupeKey);
        if (messageInBlock?.equals(messageKey)) {
          continue;
        } else {
          if (messageInBlock) {
            this.#log(
              `Previously add pending message ${messageInBlock.toString(
                'hex',
              )} at ${dupeKey.toString()}, now got ${messageKey.toString('hex')}`,
            );
          }
          void this.#tables.l1ToL2MessagesByBlock.put(dupeKey, messageKey);
        }

        const message = this.#getL1ToL2Message(messageKey);
        this.#updateMessageCountInTx(messageKey, message, -1, 0);
      }
      return true;
    });
  }

  /**
   * Messages that have been published in an L2 block are confirmed.
   * Add them to the confirmed store, also remove them from the pending store.
   * @param entryKeys - The message keys to be removed from the store.
   * @returns True if the operation is successful.
   */
  confirmL1ToL2Messages(entryKeys: Fr[]): Promise<boolean> {
    return this.#tables.l1ToL2Messages.transaction(() => {
      for (const entryKey of entryKeys) {
        const messageKey = entryKey.toBuffer();
        const message = this.#getL1ToL2Message(messageKey);
        this.#updateMessageCountInTx(messageKey, message, -1, 1);
      }
      return true;
    });
  }

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages, sorted by fee
   * @param limit - The number of messages to return (by default NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).
   * @returns The requested L1 to L2 message keys.
   */
  getPendingL1ToL2MessageKeys(limit: number): Promise<Fr[]> {
    // start a read transaction in order to have a consistent view of the data
    // this is all sync code, but better to be safe in case it changes in the future
    // or we end up having multiple processes touching the same db
    const transaction = this.#tables.pendingMessagesByFee.useReadTransaction();

    try {
      // get all the keys, in reverse order
      const fees = this.#tables.pendingMessagesByFee.getKeys({ reverse: true, transaction });
      const messages: Fr[] = [];

      loopOverFees: for (const fee of fees) {
        const pendingMessages = this.#tables.pendingMessagesByFee.getValues(fee, { transaction });
        this.#log(`Found pending messages for ${fee}`);

        for (const messageKey of pendingMessages) {
          const messageWithCount = this.#tables.l1ToL2Messages.get(messageKey, { transaction });
          if (!messageWithCount || messageWithCount.pendingCount === 0) {
            this.#log(
              `Message ${messageKey.toString(
                'hex',
              )} has no pending count but it got picked up by getPEndingL1ToL2MessageKeys`,
            );
            continue;
          }
          const toAdd = Array(messageWithCount.pendingCount).fill(Fr.fromBuffer(messageKey));
          this.#log(`Adding ${toAdd.length} copies of ${messageKey.toString('hex')} for ${fee}`);
          messages.push(...toAdd);

          if (messages.length >= limit) {
            break loopOverFees;
          }
        }
      }

      return Promise.resolve(messages);
    } catch (err) {
      return Promise.reject(err);
    } finally {
      transaction.done();
    }
  }

  /**
   * Gets the confirmed L1 to L2 message corresponding to the given message key.
   * @param messageKey - The message key to look up.
   * @returns The requested L1 to L2 message or throws if not found.
   */
  getConfirmedL1ToL2Message(messageKey: Fr): Promise<L1ToL2Message> {
    const value = this.#tables.l1ToL2Messages.get(messageKey.toBuffer());
    if (!value) {
      return Promise.reject(new Error(`Message with key ${messageKey} not found`));
    }

    if (value.confirmedCount === 0) {
      return Promise.reject(new Error(`Message with key ${messageKey} not confirmed`));
    }

    return Promise.resolve(L1ToL2Message.fromBuffer(value.message));
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
      const blockCtxKey = logType === LogType.ENCRYPTED ? 'encryptedLogs' : 'unencryptedLogs';
      const logs = this.#tables.blocks
        .getRange(this.#computeBlockRange(start, limit))
        .map(({ value: { [blockCtxKey]: logs } }) =>
          logs ? L2BlockL2Logs.fromBuffer(asBuffer(logs)) : new L2BlockL2Logs([]),
        ).asArray;

      return Promise.resolve(logs);
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
      if (filter.afterLog) {
        return Promise.resolve(this.#filterLogsBetweenBlocks(filter));
      } else if (filter.txHash) {
        return Promise.resolve(this.#filterLogsOfTx(filter));
      } else {
        return Promise.resolve(this.#filterLogsBetweenBlocks(filter));
      }
    } catch (err) {
      return Promise.reject(err);
    }
  }

  #filterLogsOfTx(filter: LogFilter): GetUnencryptedLogsResponse {
    if (!filter.txHash) {
      throw new Error('Missing txHash');
    }

    const [blockNumber, txIndex] = this.#tables.txIndex.get(filter.txHash.buffer) ?? [];
    if (typeof blockNumber !== 'number' || typeof txIndex !== 'number') {
      return { logs: [], maxLogsHit: false };
    }

    const block = this.#getBlock(blockNumber, true);
    if (!block || !block.newUnencryptedLogs) {
      return { logs: [], maxLogsHit: false };
    }

    const txLogs = block.newUnencryptedLogs.txLogs[txIndex].unrollLogs().map(log => UnencryptedL2Log.fromBuffer(log));
    const logs: ExtendedUnencryptedL2Log[] = [];
    const maxLogsHit = this.#accumulateLogs(logs, blockNumber, txIndex, txLogs, filter);

    return { logs, maxLogsHit };
  }

  #filterLogsBetweenBlocks(filter: LogFilter): GetUnencryptedLogsResponse {
    const start =
      filter.afterLog?.blockNumber ?? Math.max(filter.fromBlock ?? INITIAL_L2_BLOCK_NUM, INITIAL_L2_BLOCK_NUM);
    const end = filter.toBlock;

    if (typeof end === 'number' && end < start) {
      return {
        logs: [],
        maxLogsHit: true,
      };
    }

    const logs: ExtendedUnencryptedL2Log[] = [];

    const blockNumbers = this.#tables.blocks.getKeys({ start, end, snapshot: false });
    let maxLogsHit = false;

    loopOverBlocks: for (const blockNumber of blockNumbers) {
      const block = this.#getBlock(blockNumber, true);
      if (!block || !block.newUnencryptedLogs) {
        continue;
      }

      const unencryptedLogsInBlock = block.newUnencryptedLogs;
      for (let txIndex = filter.afterLog?.txIndex ?? 0; txIndex < unencryptedLogsInBlock.txLogs.length; txIndex++) {
        const txLogs = unencryptedLogsInBlock.txLogs[txIndex].unrollLogs().map(log => UnencryptedL2Log.fromBuffer(log));
        maxLogsHit = this.#accumulateLogs(logs, blockNumber, txIndex, txLogs, filter);
        if (maxLogsHit) {
          break loopOverBlocks;
        }
      }
    }

    return { logs, maxLogsHit };
  }

  #accumulateLogs(
    results: ExtendedUnencryptedL2Log[],
    blockNumber: number,
    txIndex: number,
    txLogs: UnencryptedL2Log[],
    filter: LogFilter,
  ): boolean {
    let maxLogsHit = false;
    let logIndex = typeof filter.afterLog?.logIndex === 'number' ? filter.afterLog.logIndex + 1 : 0;
    for (; logIndex < txLogs.length; logIndex++) {
      const log = txLogs[logIndex];
      if (filter.contractAddress && !log.contractAddress.equals(filter.contractAddress)) {
        continue;
      }

      if (filter.selector && !log.selector.equals(filter.selector)) {
        continue;
      }

      results.push(new ExtendedUnencryptedL2Log(new LogId(blockNumber, txIndex, logIndex), log));
      if (results.length >= this.#logsMaxPageSize) {
        maxLogsHit = true;
        break;
      }
    }

    return maxLogsHit;
  }

  /**
   * Add new extended contract data from an L2 block to the store's list.
   * @param data - List of contracts' data to be added.
   * @param blockNum - Number of the L2 block the contract data was deployed in.
   * @returns True if the operation is successful.
   */
  addExtendedContractData(data: ExtendedContractData[], blockNum: number): Promise<boolean> {
    return this.#tables.blocks.transaction(() => {
      const blockCtx = this.#tables.blocks.get(blockNum) ?? {};
      if (!blockCtx.extendedContractData) {
        blockCtx.extendedContractData = [];
      }
      this.#log(`Adding ${data.length} extended contract data to block ${blockNum}`);
      blockCtx.extendedContractData.push(...data.map(data => data.toBuffer()));
      void this.#tables.blocks.put(blockNum, blockCtx);

      return true;
    });
  }

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    const [blockNumber, _] = this.#tables.contractIndex.get(contractAddress.toBuffer()) ?? [];

    if (typeof blockNumber !== 'number') {
      return Promise.resolve(undefined);
    }

    const blockCtx = this.#tables.blocks.get(blockNumber);
    if (!blockCtx) {
      return Promise.resolve(undefined);
    }

    for (const data of blockCtx.extendedContractData ?? []) {
      const extendedContractData = ExtendedContractData.fromBuffer(asBuffer(data));
      if (extendedContractData.contractData.contractAddress.equals(contractAddress)) {
        return Promise.resolve(extendedContractData);
      }
    }

    return Promise.resolve(undefined);
  }

  /**
   * Lookup all extended contract data in an L2 block.
   * @param blockNum - The block number to get all contract data from.
   * @returns All extended contract data in the block (if found).
   */
  getExtendedContractDataInBlock(blockNum: number): Promise<ExtendedContractData[]> {
    const blockCtx = this.#tables.blocks.get(blockNum);
    if (!blockCtx || !blockCtx.extendedContractData) {
      return Promise.resolve([]);
    }

    return Promise.resolve(blockCtx.extendedContractData.map(data => ExtendedContractData.fromBuffer(asBuffer(data))));
  }

  /**
   * Get basic info for an L2 contract.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    const [blockNumber, index] = this.#tables.contractIndex.get(contractAddress.toBuffer()) ?? [];
    if (typeof blockNumber !== 'number' || typeof index !== 'number') {
      return Promise.resolve(undefined);
    }

    const block = this.#getBlock(blockNumber);
    return Promise.resolve(block?.newContractData[index]);
  }

  /**
   * Get basic info for an all L2 contracts deployed in a block.
   * Contains contract address & the ethereum portal address.
   * @param blockNumber - Number of the L2 block where contracts were deployed.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  getContractDataInBlock(blockNumber: number): Promise<ContractData[] | undefined> {
    const block = this.#getBlock(blockNumber);
    return Promise.resolve(block?.newContractData ?? []);
  }

  /**
   * Gets the number of the latest L2 block processed.
   * @returns The number of the latest L2 block processed.
   */
  getBlockNumber(): Promise<number> {
    // inverse range with no start/end will return the last key
    const [lastBlockNumber] = this.#tables.blocks.getKeys({ reverse: true, limit: 1 }).asArray;
    return Promise.resolve(typeof lastBlockNumber === 'number' ? lastBlockNumber : INITIAL_L2_BLOCK_NUM - 1);
  }

  getL1BlockNumber(): Promise<bigint> {
    // inverse range with no start/end will return the last value
    const [lastBlock] = this.#tables.blocks.getRange({ reverse: true, limit: 1 }).asArray;
    if (!lastBlock) {
      return Promise.resolve(0n);
    } else {
      const blockCtx = lastBlock.value;
      if (!blockCtx.l1BlockNumber) {
        return Promise.reject(new Error('L1 block number not found'));
      } else {
        return Promise.resolve(toBigIntBE(asBuffer(blockCtx.l1BlockNumber)));
      }
    }
  }

  #getBlock(blockNumber: number, withLogs = false): L2Block | undefined {
    const blockCtx = this.#tables.blocks.get(blockNumber);
    if (!blockCtx || !blockCtx.block) {
      return undefined;
    }

    const block = L2Block.fromBuffer(asBuffer(blockCtx.block));

    if (withLogs) {
      if (blockCtx.encryptedLogs) {
        block.attachLogs(L2BlockL2Logs.fromBuffer(asBuffer(blockCtx.encryptedLogs)), LogType.ENCRYPTED);
      }

      if (blockCtx.unencryptedLogs) {
        block.attachLogs(L2BlockL2Logs.fromBuffer(asBuffer(blockCtx.unencryptedLogs)), LogType.UNENCRYPTED);
      }
    }

    return block;
  }

  #computeBlockRange(start: number, limit: number): Required<Pick<RangeOptions, 'start' | 'end'>> {
    if (limit < 1) {
      throw new Error(`Invalid limit: ${limit}`);
    }

    if (start < INITIAL_L2_BLOCK_NUM) {
      this.#log(`Clamping start block ${start} to ${INITIAL_L2_BLOCK_NUM}`);
      start = INITIAL_L2_BLOCK_NUM;
    }

    const end = start + limit;
    return { start, end };
  }

  #getL1ToL2Message(entryKey: Buffer): L1ToL2Message {
    const value = this.#tables.l1ToL2Messages.get(entryKey);
    if (!value) {
      throw new Error('Unknown message: ' + entryKey.toString());
    }

    return L1ToL2Message.fromBuffer(value.message);
  }

  /**
   * Atomically updates the pending and confirmed count for a message.
   * If both counts are 0 after adding their respective deltas, the message is removed from the store.
   *
   * Only call this method from inside a _transaction_!
   *
   * @param messageKey - The message key to update.
   * @param message - The message to update.
   * @param deltaPendingCount - The amount to add to the pending count.
   * @param deltaConfirmedCount - The amount to add to the confirmed count.
   */
  #updateMessageCountInTx(
    messageKey: Buffer,
    message: L1ToL2Message,
    deltaPendingCount: number,
    deltaConfirmedCount: number,
  ): void {
    const entry = this.#tables.l1ToL2Messages.getEntry(messageKey);
    if (!entry) {
      return;
    }

    const { value } = entry;

    value.pendingCount = Math.max(0, value.pendingCount + deltaPendingCount);
    value.confirmedCount = Math.max(0, value.confirmedCount + deltaConfirmedCount);

    this.#log(
      `Updating count of ${messageKey.toString('hex')} to ${value.pendingCount} pending and ${
        value.confirmedCount
      } confirmed}`,
    );

    if (value.pendingCount === 0) {
      this.#log(`Removing message ${messageKey.toString('hex')} from pending messages group with fee ${message.fee}`);
      void this.#tables.pendingMessagesByFee.remove(message.fee, messageKey);
    } else if (value.pendingCount > 0) {
      this.#log(`Adding message ${messageKey.toString('hex')} to pending message group with fee ${message.fee}`);
      void this.#tables.pendingMessagesByFee.put(message.fee, messageKey);
    }

    if (value.pendingCount === 0 && value.confirmedCount === 0) {
      void this.#tables.l1ToL2Messages.remove(messageKey);
    } else {
      void this.#tables.l1ToL2Messages.put(messageKey, value);
    }
  }
}

/**
 * Creates a Buffer viewing the same memory location as the passed array.
 * @param arr - A Uint8Array
 */
function asBuffer(arr: Uint8Array | Buffer): Buffer {
  return Buffer.isBuffer(arr) ? arr : Buffer.from(arr.buffer, arr.byteOffset, arr.length / arr.BYTES_PER_ELEMENT);
}
