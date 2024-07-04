import {
  EncryptedL2BlockL2Logs,
  EncryptedNoteL2BlockL2Logs,
  ExtendedUnencryptedL2Log,
  type FromLogType,
  type GetUnencryptedLogsResponse,
  type L2BlockL2Logs,
  type LogFilter,
  LogId,
  LogType,
  UnencryptedL2BlockL2Logs,
  type UnencryptedL2Log,
} from '@aztec/circuit-types';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js/constants';
import { createDebugLogger } from '@aztec/foundation/log';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';

import { type BlockStore } from './block_store.js';

/**
 * A store for logs
 */
export class LogStore {
  #noteEncryptedLogs: AztecMap<number, Buffer>;
  #encryptedLogs: AztecMap<number, Buffer>;
  #unencryptedLogs: AztecMap<number, Buffer>;
  #logsMaxPageSize: number;
  #log = createDebugLogger('aztec:archiver:log_store');

  constructor(private db: AztecKVStore, private blockStore: BlockStore, logsMaxPageSize: number = 1000) {
    this.#noteEncryptedLogs = db.openMap('archiver_note_encrypted_logs');
    this.#encryptedLogs = db.openMap('archiver_encrypted_logs');
    this.#unencryptedLogs = db.openMap('archiver_unencrypted_logs');

    this.#logsMaxPageSize = logsMaxPageSize;
  }

  /**
   * Append new logs to the store's list.
   * @param encryptedLogs - The logs to be added to the store.
   * @param unencryptedLogs - The type of the logs to be added to the store.
   * @param blockNumber - The block for which to add the logs.
   * @returns True if the operation is successful.
   */
  addLogs(
    noteEncryptedLogs: EncryptedNoteL2BlockL2Logs | undefined,
    encryptedLogs: EncryptedL2BlockL2Logs | undefined,
    unencryptedLogs: UnencryptedL2BlockL2Logs | undefined,
    blockNumber: number,
  ): Promise<boolean> {
    return this.db.transaction(() => {
      if (noteEncryptedLogs) {
        void this.#noteEncryptedLogs.set(blockNumber, noteEncryptedLogs.toBuffer());
      }

      if (encryptedLogs) {
        void this.#encryptedLogs.set(blockNumber, encryptedLogs.toBuffer());
      }

      if (unencryptedLogs) {
        void this.#unencryptedLogs.set(blockNumber, unencryptedLogs.toBuffer());
      }

      return true;
    });
  }

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param start - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  *getLogs<TLogType extends LogType>(
    start: number,
    limit: number,
    logType: TLogType,
  ): IterableIterator<L2BlockL2Logs<FromLogType<TLogType>>> {
    const logMap = (() => {
      switch (logType) {
        case LogType.ENCRYPTED:
          return this.#encryptedLogs;
        case LogType.NOTEENCRYPTED:
          return this.#noteEncryptedLogs;
        case LogType.UNENCRYPTED:
        default:
          return this.#unencryptedLogs;
      }
    })();
    const logTypeMap = (() => {
      switch (logType) {
        case LogType.ENCRYPTED:
          return EncryptedL2BlockL2Logs;
        case LogType.NOTEENCRYPTED:
          return EncryptedNoteL2BlockL2Logs;
        case LogType.UNENCRYPTED:
        default:
          return UnencryptedL2BlockL2Logs;
      }
    })();
    const L2BlockL2Logs = logTypeMap;
    for (const buffer of logMap.values({ start, limit })) {
      yield L2BlockL2Logs.fromBuffer(buffer) as L2BlockL2Logs<FromLogType<TLogType>>;
    }
  }

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  getUnencryptedLogs(filter: LogFilter): GetUnencryptedLogsResponse {
    if (filter.afterLog) {
      return this.#filterUnencryptedLogsBetweenBlocks(filter);
    } else if (filter.txHash) {
      return this.#filterUnencryptedLogsOfTx(filter);
    } else {
      return this.#filterUnencryptedLogsBetweenBlocks(filter);
    }
  }

  #filterUnencryptedLogsOfTx(filter: LogFilter): GetUnencryptedLogsResponse {
    if (!filter.txHash) {
      throw new Error('Missing txHash');
    }

    const [blockNumber, txIndex] = this.blockStore.getTxLocation(filter.txHash) ?? [];
    if (typeof blockNumber !== 'number' || typeof txIndex !== 'number') {
      return { logs: [], maxLogsHit: false };
    }

    const unencryptedLogsInBlock = this.#getBlockLogs(blockNumber, LogType.UNENCRYPTED);
    const txLogs = unencryptedLogsInBlock.txLogs[txIndex].unrollLogs();

    const logs: ExtendedUnencryptedL2Log[] = [];
    const maxLogsHit = this.#accumulateLogs(logs, blockNumber, txIndex, txLogs, filter);

    return { logs, maxLogsHit };
  }

  #filterUnencryptedLogsBetweenBlocks(filter: LogFilter): GetUnencryptedLogsResponse {
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

    let maxLogsHit = false;
    loopOverBlocks: for (const [blockNumber, logBuffer] of this.#unencryptedLogs.entries({ start, end })) {
      const unencryptedLogsInBlock = UnencryptedL2BlockL2Logs.fromBuffer(logBuffer);
      for (let txIndex = filter.afterLog?.txIndex ?? 0; txIndex < unencryptedLogsInBlock.txLogs.length; txIndex++) {
        const txLogs = unencryptedLogsInBlock.txLogs[txIndex].unrollLogs();
        maxLogsHit = this.#accumulateLogs(logs, blockNumber, txIndex, txLogs, filter);
        if (maxLogsHit) {
          this.#log.debug(`Max logs hit at block ${blockNumber}`);
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
      if (!filter.contractAddress || log.contractAddress.equals(filter.contractAddress)) {
        results.push(new ExtendedUnencryptedL2Log(new LogId(blockNumber, txIndex, logIndex), log));
        if (results.length >= this.#logsMaxPageSize) {
          maxLogsHit = true;
          break;
        }
      }
    }

    return maxLogsHit;
  }

  #getBlockLogs<TLogType extends LogType>(
    blockNumber: number,
    logType: TLogType,
  ): L2BlockL2Logs<FromLogType<TLogType>> {
    const logMap = (() => {
      switch (logType) {
        case LogType.ENCRYPTED:
          return this.#encryptedLogs;
        case LogType.NOTEENCRYPTED:
          return this.#noteEncryptedLogs;
        case LogType.UNENCRYPTED:
        default:
          return this.#unencryptedLogs;
      }
    })();
    const logTypeMap = (() => {
      switch (logType) {
        case LogType.ENCRYPTED:
          return EncryptedL2BlockL2Logs;
        case LogType.NOTEENCRYPTED:
          return EncryptedNoteL2BlockL2Logs;
        case LogType.UNENCRYPTED:
        default:
          return UnencryptedL2BlockL2Logs;
      }
    })();
    const L2BlockL2Logs = logTypeMap;
    const buffer = logMap.get(blockNumber);

    if (!buffer) {
      return new L2BlockL2Logs([]) as L2BlockL2Logs<FromLogType<TLogType>>;
    }

    return L2BlockL2Logs.fromBuffer(buffer) as L2BlockL2Logs<FromLogType<TLogType>>;
  }
}
