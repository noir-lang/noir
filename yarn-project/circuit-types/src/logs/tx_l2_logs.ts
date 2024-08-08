import {
  Fr,
  type LogHash,
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
  type ScopedLogHash,
} from '@aztec/circuits.js';
import { sha256Trunc } from '@aztec/foundation/crypto';
import { BufferReader, prefixBufferWithLength } from '@aztec/foundation/serialize';

import isEqual from 'lodash.isequal';

import { type EncryptedL2Log } from './encrypted_l2_log.js';
import { type EncryptedL2NoteLog } from './encrypted_l2_note_log.js';
import {
  EncryptedFunctionL2Logs,
  EncryptedNoteFunctionL2Logs,
  type FunctionL2Logs,
  UnencryptedFunctionL2Logs,
} from './function_l2_logs.js';
import { type UnencryptedL2Log } from './unencrypted_l2_log.js';

/**
 * Data container of logs emitted in 1 tx.
 */
export abstract class TxL2Logs<TLog extends UnencryptedL2Log | EncryptedL2NoteLog | EncryptedL2Log> {
  abstract hash(): Buffer;

  constructor(
    /** * An array containing logs emitted in individual function invocations in this tx. */
    public readonly functionLogs: FunctionL2Logs<TLog>[],
  ) {}

  /**
   * Serializes logs into a buffer.
   * @returns A buffer containing the serialized logs.
   */
  public toBuffer(): Buffer {
    const serializedFunctionLogs = this.functionLogs.map(logs => logs.toBuffer());
    // Concatenate all serialized function logs into a single buffer and prefix it with 4 bytes for its total length.
    return prefixBufferWithLength(Buffer.concat(serializedFunctionLogs));
  }

  /**
   * Get the total length of serialized data.
   * @returns Total length of serialized data.
   */
  public getSerializedLength(): number {
    return this.functionLogs.reduce((acc, logs) => acc + logs.getSerializedLength(), 0) + 4;
  }

  /**
   * Get the total length of all chargable data (raw log data + 4 for each log)
   * TODO: Rename this? getChargableLength? getDALength?
   * @returns Total length of data.
   */
  public getKernelLength(): number {
    return this.functionLogs.reduce((acc, logs) => acc + logs.getKernelLength(), 0);
  }

  /** Gets the total number of logs. */
  public getTotalLogCount() {
    return this.functionLogs.reduce((acc, logs) => acc + logs.logs.length, 0);
  }

  /**
   * Adds function logs to the existing logs.
   * @param functionLogs - The function logs to add
   * @remarks Used by sequencer to append unencrypted logs emitted in public function calls.
   */
  public addFunctionLogs(functionLogs: FunctionL2Logs<TLog>[]) {
    this.functionLogs.push(...functionLogs);
  }

  /**
   * Convert a TxL2Logs class object to a plain JSON object.
   * @returns A plain object with TxL2Logs properties.
   */
  public toJSON() {
    return {
      functionLogs: this.functionLogs.map(log => log.toJSON()),
    };
  }

  /**
   * Unrolls logs from this tx.
   * @returns Unrolled logs.
   */
  public unrollLogs(): TLog[] {
    return this.functionLogs.flatMap(functionLog => functionLog.logs);
  }

  /**
   * Checks if two TxL2Logs objects are equal.
   * @param other - Another TxL2Logs object to compare with.
   * @returns True if the two objects are equal, false otherwise.
   */
  public equals(other: TxL2Logs<TLog>): boolean {
    return isEqual(this, other);
  }

  /**
   * Filter the logs from functions from this TxL2Logs that
   * appear in the provided logHashes
   * @param logHashes hashes we want to keep
   * @param output our aggregation
   * @returns our aggregation
   */
  public filter(logHashes: LogHash[], output: TxL2Logs<TLog>): TxL2Logs<TLog> {
    for (const fnLogs of this.functionLogs) {
      let include = false;
      for (const log of fnLogs.logs) {
        if (logHashes.findIndex(lh => lh.value.equals(Fr.fromBuffer(log.getSiloedHash()))) !== -1) {
          include = true;
        }
      }
      if (include) {
        output.addFunctionLogs([fnLogs]);
      }
    }
    return output;
  }

  /**
   * Filter the logs from functions from this TxL2Logs that
   * appear in the provided scopedLogHashes
   * @param logHashes hashes we want to keep
   * @param output our aggregation
   * @returns our aggregation
   */
  public filterScoped(scopedLogHashes: ScopedLogHash[], output: TxL2Logs<TLog>): TxL2Logs<TLog> {
    for (const fnLogs of this.functionLogs) {
      let include = false;
      for (const log of fnLogs.logs) {
        let contractAddress;
        if ('contractAddress' in log) {
          contractAddress = log.contractAddress;
        } else if ('maskedContractAddress' in log) {
          contractAddress = log.maskedContractAddress;
        } else {
          throw new Error("Can't run filterScoped in logs without contractAddress or maskedContractAddress");
        }
        if (
          scopedLogHashes.findIndex(
            slh => slh.contractAddress.equals(contractAddress) && slh.value.equals(Fr.fromBuffer(log.hash())),
          ) != -1
        ) {
          include = true;
        }
      }
      if (include) {
        output.addFunctionLogs([fnLogs]);
      }
    }
    return output;
  }
}

export class UnencryptedTxL2Logs extends TxL2Logs<UnencryptedL2Log> {
  /** Creates an empty instance. */
  public static empty() {
    return new UnencryptedTxL2Logs([]);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns A new L2Logs object.
   */
  public static fromBuffer(buf: Buffer | BufferReader, isLengthPrefixed = true): UnencryptedTxL2Logs {
    const reader = BufferReader.asReader(buf);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const serializedFunctionLogs = reader.readBufferArray(logsBufLength);

    const functionLogs = serializedFunctionLogs.map(logs => UnencryptedFunctionL2Logs.fromBuffer(logs, false));
    return new UnencryptedTxL2Logs(functionLogs);
  }

  /**
   * Creates a new `TxL2Logs` object with `numCalls` function logs and `numLogsPerCall` logs in each invocation.
   * @param numCalls - The number of function calls in the tx.
   * @param numLogsPerCall - The number of logs emitted in each function call.
   * @param logType - The type of logs to generate.
   * @returns A new `TxL2Logs` object.
   */
  public static random(numCalls: number, numLogsPerCall: number): UnencryptedTxL2Logs {
    if (numCalls * numLogsPerCall > MAX_UNENCRYPTED_LOGS_PER_TX) {
      throw new Error(
        `Trying to create ${numCalls * numLogsPerCall} logs for one tx (max: ${MAX_UNENCRYPTED_LOGS_PER_TX})`,
      );
    }
    const functionLogs: UnencryptedFunctionL2Logs[] = [];
    for (let i = 0; i < numCalls; i++) {
      functionLogs.push(UnencryptedFunctionL2Logs.random(numLogsPerCall));
    }
    return new UnencryptedTxL2Logs(functionLogs);
  }

  /**
   * Convert a plain JSON object to a TxL2Logs class object.
   * @param obj - A plain TxL2Logs JSON object.
   * @returns A TxL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const functionLogs = obj.functionLogs.map((log: any) => UnencryptedFunctionL2Logs.fromJSON(log));
    return new UnencryptedTxL2Logs(functionLogs);
  }

  /**
   * Computes unencrypted logs hash as is done in the kernel and decoder contract.
   * @param logs - Logs to be hashed.
   * @returns The hash of the logs.
   * Note: This is a TS implementation of `computeKernelUnencryptedLogsHash` function in Decoder.sol. See that function documentation
   *       for more details.
   */
  public override hash(): Buffer {
    const unrolledLogs = this.unrollLogs();
    return UnencryptedTxL2Logs.hashSiloedLogs(unrolledLogs.map(log => log.getSiloedHash()));
  }

  /**
   * Hashes siloed unencrypted logs as in the same way as the base rollup would.
   * @param siloedLogHashes - The siloed log hashes
   * @returns The hash of the logs.
   */
  public static hashSiloedLogs(siloedLogHashes: Buffer[]): Buffer {
    if (siloedLogHashes.length == 0) {
      return Buffer.alloc(32);
    }

    let allSiloedLogHashes = Buffer.alloc(0);
    for (const siloedLogHash of siloedLogHashes) {
      allSiloedLogHashes = Buffer.concat([allSiloedLogHashes, siloedLogHash]);
    }
    // pad the end of logs with 0s
    for (let i = 0; i < MAX_UNENCRYPTED_LOGS_PER_TX - siloedLogHashes.length; i++) {
      allSiloedLogHashes = Buffer.concat([allSiloedLogHashes, Buffer.alloc(32)]);
    }

    return sha256Trunc(allSiloedLogHashes);
  }
}

export class EncryptedNoteTxL2Logs extends TxL2Logs<EncryptedL2NoteLog> {
  /** Creates an empty instance. */
  public static empty() {
    return new EncryptedNoteTxL2Logs([]);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns A new L2Logs object.
   */
  public static fromBuffer(buf: Buffer | BufferReader, isLengthPrefixed = true): EncryptedNoteTxL2Logs {
    const reader = BufferReader.asReader(buf);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const serializedFunctionLogs = reader.readBufferArray(logsBufLength);

    const functionLogs = serializedFunctionLogs.map(logs => EncryptedNoteFunctionL2Logs.fromBuffer(logs, false));
    return new EncryptedNoteTxL2Logs(functionLogs);
  }

  /**
   * Creates a new `TxL2Logs` object with `numCalls` function logs and `numLogsPerCall` logs in each invocation.
   * @param numCalls - The number of function calls in the tx.
   * @param numLogsPerCall - The number of logs emitted in each function call.
   * @param logType - The type of logs to generate.
   * @returns A new `TxL2Logs` object.
   */
  public static random(numCalls: number, numLogsPerCall: number): EncryptedNoteTxL2Logs {
    if (numCalls * numLogsPerCall > MAX_NOTE_ENCRYPTED_LOGS_PER_TX) {
      throw new Error(
        `Trying to create ${numCalls * numLogsPerCall} logs for one tx (max: ${MAX_NOTE_ENCRYPTED_LOGS_PER_TX})`,
      );
    }
    const functionLogs: EncryptedNoteFunctionL2Logs[] = [];
    for (let i = 0; i < numCalls; i++) {
      functionLogs.push(EncryptedNoteFunctionL2Logs.random(numLogsPerCall));
    }
    return new EncryptedNoteTxL2Logs(functionLogs);
  }

  /**
   * Convert a plain JSON object to a TxL2Logs class object.
   * @param obj - A plain TxL2Logs JSON object.
   * @returns A TxL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const functionLogs = obj.functionLogs.map((log: any) => EncryptedNoteFunctionL2Logs.fromJSON(log));
    return new EncryptedNoteTxL2Logs(functionLogs);
  }

  /**
   * Computes encrypted logs hash as is done in the kernel and decoder contract.
   * @param logs - Logs to be hashed.
   * @returns The hash of the logs.
   * Note: This is a TS implementation of `computeKernelNoteEncryptedLogsHash` function in Decoder.sol. See that function documentation
   *       for more details.
   */
  public override hash(): Buffer {
    return EncryptedNoteTxL2Logs.hashNoteLogs(this.unrollLogs().map(log => log.hash()));
  }

  /**
   * Hashes encrypted note logs hashes as in the same way as the base rollup would.
   * @param siloedLogHashes - The note log hashes
   * @returns The hash of the log hashes.
   */
  public static hashNoteLogs(logHashes: Buffer[]): Buffer {
    if (logHashes.length == 0) {
      return Buffer.alloc(32);
    }

    let allSiloedLogHashes = Buffer.alloc(0);
    for (const siloedLogHash of logHashes) {
      allSiloedLogHashes = Buffer.concat([allSiloedLogHashes, siloedLogHash]);
    }
    // pad the end of logs with 0s
    for (let i = 0; i < MAX_NOTE_ENCRYPTED_LOGS_PER_TX - logHashes.length; i++) {
      allSiloedLogHashes = Buffer.concat([allSiloedLogHashes, Buffer.alloc(32)]);
    }

    return sha256Trunc(allSiloedLogHashes);
  }
}

export class EncryptedTxL2Logs extends TxL2Logs<EncryptedL2Log> {
  /** Creates an empty instance. */
  public static empty() {
    return new EncryptedTxL2Logs([]);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns A new L2Logs object.
   */
  public static fromBuffer(buf: Buffer | BufferReader, isLengthPrefixed = true): EncryptedTxL2Logs {
    const reader = BufferReader.asReader(buf);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const serializedFunctionLogs = reader.readBufferArray(logsBufLength);

    const functionLogs = serializedFunctionLogs.map(logs => EncryptedFunctionL2Logs.fromBuffer(logs, false));
    return new EncryptedTxL2Logs(functionLogs);
  }

  /**
   * Creates a new `TxL2Logs` object with `numCalls` function logs and `numLogsPerCall` logs in each invocation.
   * @param numCalls - The number of function calls in the tx.
   * @param numLogsPerCall - The number of logs emitted in each function call.
   * @param logType - The type of logs to generate.
   * @returns A new `TxL2Logs` object.
   */
  public static random(numCalls: number, numLogsPerCall: number): EncryptedTxL2Logs {
    if (numCalls * numLogsPerCall > MAX_ENCRYPTED_LOGS_PER_TX) {
      throw new Error(
        `Trying to create ${numCalls * numLogsPerCall} logs for one tx (max: ${MAX_ENCRYPTED_LOGS_PER_TX})`,
      );
    }
    const functionLogs: EncryptedFunctionL2Logs[] = [];
    for (let i = 0; i < numCalls; i++) {
      functionLogs.push(EncryptedFunctionL2Logs.random(numLogsPerCall));
    }
    return new EncryptedTxL2Logs(functionLogs);
  }

  /**
   * Convert a plain JSON object to a TxL2Logs class object.
   * @param obj - A plain TxL2Logs JSON object.
   * @returns A TxL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const functionLogs = obj.functionLogs.map((log: any) => EncryptedFunctionL2Logs.fromJSON(log));
    return new EncryptedTxL2Logs(functionLogs);
  }

  /**
   * Computes encrypted logs hash as is done in the kernel and decoder contract.
   * @param logs - Logs to be hashed.
   * @returns The hash of the logs.
   * Note: This is a TS implementation of `computeKernelEncryptedLogsHash` function in Decoder.sol. See that function documentation
   *       for more details.
   */
  public override hash(): Buffer {
    const unrolledLogs = this.unrollLogs();
    return EncryptedTxL2Logs.hashSiloedLogs(unrolledLogs.map(log => log.getSiloedHash()));
  }

  /**
   * Hashes siloed unencrypted logs as in the same way as the base rollup would.
   * @param siloedLogHashes - The siloed log hashes
   * @returns The hash of the logs.
   */
  public static hashSiloedLogs(siloedLogHashes: Buffer[]): Buffer {
    if (siloedLogHashes.length == 0) {
      return Buffer.alloc(32);
    }

    let allSiloedLogHashes = Buffer.alloc(0);
    for (const siloedLogHash of siloedLogHashes) {
      allSiloedLogHashes = Buffer.concat([allSiloedLogHashes, siloedLogHash]);
    }
    // pad the end of logs with 0s
    for (let i = 0; i < MAX_UNENCRYPTED_LOGS_PER_TX - siloedLogHashes.length; i++) {
      allSiloedLogHashes = Buffer.concat([allSiloedLogHashes, Buffer.alloc(32)]);
    }

    return sha256Trunc(allSiloedLogHashes);
  }
}
