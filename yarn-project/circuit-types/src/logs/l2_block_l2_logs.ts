import { BufferReader, prefixBufferWithLength } from '@aztec/foundation/serialize';

import isEqual from 'lodash.isequal';

import { type EncryptedL2Log } from './encrypted_l2_log.js';
import { type EncryptedL2NoteLog } from './encrypted_l2_note_log.js';
import { EncryptedNoteTxL2Logs, EncryptedTxL2Logs, type TxL2Logs, UnencryptedTxL2Logs } from './tx_l2_logs.js';
import { type UnencryptedL2Log } from './unencrypted_l2_log.js';

/**
 * Data container of logs emitted in all txs in a given L2 block.
 */
export abstract class L2BlockL2Logs<TLog extends UnencryptedL2Log | EncryptedL2NoteLog | EncryptedL2Log> {
  constructor(
    /**
     * An array containing logs emitted in individual function invocations in this tx.
     */
    public readonly txLogs: TxL2Logs<TLog>[],
  ) {}

  /**
   * Serializes logs into a buffer.
   * @returns A buffer containing the serialized logs.
   */
  public toBuffer(): Buffer {
    const serializedTxLogs = this.txLogs.map(logs => logs.toBuffer());
    // Concatenate all serialized function logs into a single buffer and prefix it with 4 bytes for its total length.
    return prefixBufferWithLength(Buffer.concat(serializedTxLogs));
  }

  /**
   * Get the total length of serialized data.
   * @returns Total length of serialized data.
   */
  public getSerializedLength(): number {
    return this.txLogs.reduce((acc, logs) => acc + logs.getSerializedLength(), 0) + 4;
  }

  /**
   * Gets the total number of logs emitted from all the TxL2Logs.
   */
  public getTotalLogCount(): number {
    return this.txLogs.reduce((acc, logs) => acc + logs.getTotalLogCount(), 0);
  }

  /**
   * Seralizes logs into a string.
   * @returns A string representation of the serialized logs.
   */
  public toString(): string {
    return this.toBuffer().toString('hex');
  }

  /**
   * Convert a L2BlockL2Logs class object to a plain JSON object.
   * @returns A plain object with L2BlockL2Logs properties.
   */
  public toJSON() {
    return {
      txLogs: this.txLogs.map(log => log.toJSON()),
    };
  }

  /**
   * Checks if two L2BlockL2Logs objects are equal.
   * @param other - Another L2BlockL2Logs object to compare with.
   * @returns True if the two objects are equal, false otherwise.
   */
  public equals(other: L2BlockL2Logs<TLog>): boolean {
    return isEqual(this, other);
  }

  /**
   * Returns the total number of log entries across an array of L2BlockL2Logs.
   * @param l2BlockL2logs - L2BlockL2Logs to sum over.
   * @returns Total sum of log entries.
   */
  public static getTotalLogCount<TLog extends UnencryptedL2Log | EncryptedL2NoteLog | EncryptedL2Log>(
    l2BlockL2logs: L2BlockL2Logs<TLog>[],
  ): number {
    return l2BlockL2logs.reduce((sum, log) => sum + log.getTotalLogCount(), 0);
  }
}

export class EncryptedNoteL2BlockL2Logs extends L2BlockL2Logs<EncryptedL2NoteLog> {
  /**
   * Convert a plain JSON object to a L2BlockL2Logs class object.
   * @param obj - A plain L2BlockL2Logs JSON object.
   * @returns A L2BlockL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const txLogs = obj.txLogs.map((log: any) => EncryptedNoteTxL2Logs.fromJSON(log));
    return new EncryptedNoteL2BlockL2Logs(txLogs);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buffer - The buffer containing the serialized logs.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): EncryptedNoteL2BlockL2Logs {
    const reader = BufferReader.asReader(buffer);

    const logsBufLength = reader.readNumber();
    const serializedTxLogs = reader.readBufferArray(logsBufLength);

    const txLogs = serializedTxLogs.map(logs => EncryptedNoteTxL2Logs.fromBuffer(logs, false));
    return new EncryptedNoteL2BlockL2Logs(txLogs);
  }

  /**
   * Deserializes logs from a string.
   * @param data - The string containing the serialized logs.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static fromString(data: string): EncryptedNoteL2BlockL2Logs {
    const buffer = Buffer.from(data, 'hex');
    return EncryptedNoteL2BlockL2Logs.fromBuffer(buffer);
  }

  /**
   * Creates a new `L2BlockL2Logs` object with `numCalls` function logs and `numLogsPerCall` logs in each function
   * call.
   * @param numTxs - The number of txs in the block.
   * @param numCalls - The number of function calls in the tx.
   * @param numLogsPerCall - The number of logs emitted in each function call.
   * @param logType - The type of logs to generate.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static random(numTxs: number, numCalls: number, numLogsPerCall: number): EncryptedNoteL2BlockL2Logs {
    const txLogs: EncryptedNoteTxL2Logs[] = [];
    for (let i = 0; i < numTxs; i++) {
      txLogs.push(EncryptedNoteTxL2Logs.random(numCalls, numLogsPerCall));
    }
    return new EncryptedNoteL2BlockL2Logs(txLogs);
  }

  /**
   * Unrolls logs from a set of blocks.
   * @param blockLogs - Input logs from a set of blocks.
   * @returns Unrolled logs.
   */
  public static unrollLogs(blockLogs: (EncryptedNoteL2BlockL2Logs | undefined)[]): EncryptedL2NoteLog[] {
    const logs: EncryptedL2NoteLog[] = [];
    for (const blockLog of blockLogs) {
      if (blockLog) {
        for (const txLog of blockLog.txLogs) {
          logs.push(...txLog.unrollLogs());
        }
      }
    }
    return logs;
  }
}

export class EncryptedL2BlockL2Logs extends L2BlockL2Logs<EncryptedL2Log> {
  /**
   * Convert a plain JSON object to a L2BlockL2Logs class object.
   * @param obj - A plain L2BlockL2Logs JSON object.
   * @returns A L2BlockL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const txLogs = obj.txLogs.map((log: any) => EncryptedTxL2Logs.fromJSON(log));
    return new EncryptedL2BlockL2Logs(txLogs);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buffer - The buffer containing the serialized logs.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): EncryptedL2BlockL2Logs {
    const reader = BufferReader.asReader(buffer);

    const logsBufLength = reader.readNumber();
    const serializedTxLogs = reader.readBufferArray(logsBufLength);

    const txLogs = serializedTxLogs.map(logs => EncryptedTxL2Logs.fromBuffer(logs, false));
    return new EncryptedL2BlockL2Logs(txLogs);
  }

  /**
   * Deserializes logs from a string.
   * @param data - The string containing the serialized logs.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static fromString(data: string): EncryptedL2BlockL2Logs {
    const buffer = Buffer.from(data, 'hex');
    return EncryptedL2BlockL2Logs.fromBuffer(buffer);
  }

  /**
   * Creates a new `L2BlockL2Logs` object with `numCalls` function logs and `numLogsPerCall` logs in each function
   * call.
   * @param numTxs - The number of txs in the block.
   * @param numCalls - The number of function calls in the tx.
   * @param numLogsPerCall - The number of logs emitted in each function call.
   * @param logType - The type of logs to generate.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static random(numTxs: number, numCalls: number, numLogsPerCall: number): EncryptedL2BlockL2Logs {
    const txLogs: EncryptedTxL2Logs[] = [];
    for (let i = 0; i < numTxs; i++) {
      txLogs.push(EncryptedTxL2Logs.random(numCalls, numLogsPerCall));
    }
    return new EncryptedL2BlockL2Logs(txLogs);
  }

  /**
   * Unrolls logs from a set of blocks.
   * @param blockLogs - Input logs from a set of blocks.
   * @returns Unrolled logs.
   */
  public static unrollLogs(blockLogs: (EncryptedL2BlockL2Logs | undefined)[]): EncryptedL2Log[] {
    const logs: EncryptedL2Log[] = [];
    for (const blockLog of blockLogs) {
      if (blockLog) {
        for (const txLog of blockLog.txLogs) {
          logs.push(...txLog.unrollLogs());
        }
      }
    }
    return logs;
  }
}

export class UnencryptedL2BlockL2Logs extends L2BlockL2Logs<UnencryptedL2Log> {
  /**
   * Convert a plain JSON object to a L2BlockL2Logs class object.
   * @param obj - A plain L2BlockL2Logs JSON object.
   * @returns A L2BlockL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const txLogs = obj.txLogs.map((log: any) => UnencryptedTxL2Logs.fromJSON(log));
    return new UnencryptedL2BlockL2Logs(txLogs);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buffer - The buffer containing the serialized logs.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): UnencryptedL2BlockL2Logs {
    const reader = BufferReader.asReader(buffer);

    const logsBufLength = reader.readNumber();
    const serializedTxLogs = reader.readBufferArray(logsBufLength);

    const txLogs = serializedTxLogs.map(logs => UnencryptedTxL2Logs.fromBuffer(logs, false));
    return new UnencryptedL2BlockL2Logs(txLogs);
  }

  /**
   * Deserializes logs from a string.
   * @param data - The string containing the serialized logs.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static fromString(data: string): UnencryptedL2BlockL2Logs {
    const buffer = Buffer.from(data, 'hex');
    return UnencryptedL2BlockL2Logs.fromBuffer(buffer);
  }

  /**
   * Creates a new `L2BlockL2Logs` object with `numCalls` function logs and `numLogsPerCall` logs in each function
   * call.
   * @param numTxs - The number of txs in the block.
   * @param numCalls - The number of function calls in the tx.
   * @param numLogsPerCall - The number of logs emitted in each function call.
   * @param logType - The type of logs to generate.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static random(numTxs: number, numCalls: number, numLogsPerCall: number): UnencryptedL2BlockL2Logs {
    const txLogs: UnencryptedTxL2Logs[] = [];
    for (let i = 0; i < numTxs; i++) {
      txLogs.push(UnencryptedTxL2Logs.random(numCalls, numLogsPerCall));
    }
    return new UnencryptedL2BlockL2Logs(txLogs);
  }

  /**
   * Unrolls logs from a set of blocks.
   * @param blockLogs - Input logs from a set of blocks.
   * @returns Unrolled logs.
   */
  public static unrollLogs(blockLogs: (UnencryptedL2BlockL2Logs | undefined)[]): UnencryptedL2Log[] {
    const logs: UnencryptedL2Log[] = [];
    for (const blockLog of blockLogs) {
      if (blockLog) {
        for (const txLog of blockLog.txLogs) {
          logs.push(...txLog.unrollLogs());
        }
      }
    }
    return logs;
  }
}
