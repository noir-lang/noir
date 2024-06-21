import {
  MAX_ENCRYPTED_LOGS_PER_CALL,
  MAX_NOTE_ENCRYPTED_LOGS_PER_CALL,
  MAX_UNENCRYPTED_LOGS_PER_CALL,
} from '@aztec/circuits.js';
import { sha256Trunc } from '@aztec/foundation/crypto';
import { BufferReader, prefixBufferWithLength } from '@aztec/foundation/serialize';

import { EncryptedL2Log } from './encrypted_l2_log.js';
import { EncryptedL2NoteLog } from './encrypted_l2_note_log.js';
import { UnencryptedL2Log } from './unencrypted_l2_log.js';

/**
 * Data container of logs emitted in 1 function invocation (corresponds to 1 kernel iteration).
 */
export abstract class FunctionL2Logs<TLog extends UnencryptedL2Log | EncryptedL2NoteLog | EncryptedL2Log> {
  constructor(
    /**
     * An array of logs.
     */
    public readonly logs: TLog[],
  ) {}

  /**
   * Serializes all function logs into a buffer.
   * @returns A buffer containing the serialized logs.
   * @remarks Each log is prefixed with 4 bytes for its length, then all the serialized logs are concatenated and
   *          the resulting buffer is prefixed with 4 bytes for its total length.
   */
  public toBuffer(): Buffer {
    const serializedLogs = this.logs.map(log => prefixBufferWithLength(log.toBuffer()));
    return prefixBufferWithLength(Buffer.concat(serializedLogs));
  }

  /**
   * Get the total length of all serialized data
   * @returns Total length of serialized data.
   */
  public getSerializedLength(): number {
    // adding 4 for the resulting buffer length.
    return this.getKernelLength() + 4;
  }

  /**
   * Get the total length of all chargable data (raw log data + 4 for each log)
   * TODO: Rename this? getChargableLength? getDALength?
   * @returns Total length of data.
   */
  public getKernelLength(): number {
    // Adding 4 to each log's length to account for the size stored in the serialized buffer
    return this.logs.reduce((acc, log) => acc + log.length + 4, 0);
  }

  /**
   * Calculates hash of serialized logs.
   * @returns Buffer containing 248 bits of information of sha256 hash.
   */
  public hash(): Buffer {
    // Truncated SHA hash of the concatenation of the hash of each inner log
    // Changed in resolving #5017 to mimic logs hashing in kernels
    const preimage = Buffer.concat(this.logs.map(l => l.hash()));
    return sha256Trunc(preimage);
  }

  /**
   * Convert a FunctionL2Logs class object to a plain JSON object.
   * @returns A plain object with FunctionL2Logs properties.
   */
  public toJSON() {
    return {
      logs: this.logs.map(log => log.toJSON()),
    };
  }
}

export class EncryptedNoteFunctionL2Logs extends FunctionL2Logs<EncryptedL2NoteLog> {
  /**
   * Creates an empty L2Logs object with no logs.
   * @returns A new FunctionL2Logs object with no logs.
   */
  public static empty(): EncryptedNoteFunctionL2Logs {
    return new EncryptedNoteFunctionL2Logs([]);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns Deserialized instance of `FunctionL2Logs`.
   */
  public static fromBuffer(buf: Buffer, isLengthPrefixed = true): EncryptedNoteFunctionL2Logs {
    const reader = new BufferReader(buf, 0);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const logs = reader.readBufferArray(logsBufLength);

    return new EncryptedNoteFunctionL2Logs(logs.map(EncryptedL2NoteLog.fromBuffer));
  }

  /**
   * Creates a new L2Logs object with `numLogs` logs.
   * @param numLogs - The number of logs to create.
   * @returns A new EncryptedNoteFunctionL2Logs object.
   */
  public static random(numLogs: number): EncryptedNoteFunctionL2Logs {
    if (numLogs > MAX_NOTE_ENCRYPTED_LOGS_PER_CALL) {
      throw new Error(`Trying to create ${numLogs} logs for one call (max: ${MAX_NOTE_ENCRYPTED_LOGS_PER_CALL})`);
    }
    const logs: EncryptedL2NoteLog[] = [];
    for (let i = 0; i < numLogs; i++) {
      logs.push(EncryptedL2NoteLog.random());
    }
    return new EncryptedNoteFunctionL2Logs(logs);
  }

  /**
   * Convert a plain JSON object to a FunctionL2Logs class object.
   * @param obj - A plain FunctionL2Logs JSON object.
   * @returns A FunctionL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const logs = obj.logs.map(EncryptedL2NoteLog.fromJSON);
    return new EncryptedNoteFunctionL2Logs(logs);
  }
}

export class EncryptedFunctionL2Logs extends FunctionL2Logs<EncryptedL2Log> {
  /**
   * Creates an empty L2Logs object with no logs.
   * @returns A new FunctionL2Logs object with no logs.
   */
  public static empty(): EncryptedFunctionL2Logs {
    return new EncryptedFunctionL2Logs([]);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns Deserialized instance of `FunctionL2Logs`.
   */
  public static fromBuffer(buf: Buffer, isLengthPrefixed = true): EncryptedFunctionL2Logs {
    const reader = new BufferReader(buf, 0);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const logs = reader.readBufferArray(logsBufLength);

    return new EncryptedFunctionL2Logs(logs.map(EncryptedL2Log.fromBuffer));
  }

  /**
   * Creates a new L2Logs object with `numLogs` logs.
   * @param numLogs - The number of logs to create.
   * @returns A new EncryptedFunctionL2Logs object.
   */
  public static random(numLogs: number): EncryptedFunctionL2Logs {
    if (numLogs > MAX_ENCRYPTED_LOGS_PER_CALL) {
      throw new Error(`Trying to create ${numLogs} logs for one call (max: ${MAX_ENCRYPTED_LOGS_PER_CALL})`);
    }
    const logs: EncryptedL2Log[] = [];
    for (let i = 0; i < numLogs; i++) {
      logs.push(EncryptedL2Log.random());
    }
    return new EncryptedFunctionL2Logs(logs);
  }

  /**
   * Convert a plain JSON object to a FunctionL2Logs class object.
   * @param obj - A plain FunctionL2Logs JSON object.
   * @returns A FunctionL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const logs = obj.logs.map(EncryptedL2Log.fromJSON);
    return new EncryptedFunctionL2Logs(logs);
  }
}

export class UnencryptedFunctionL2Logs extends FunctionL2Logs<UnencryptedL2Log> {
  /**
   * Creates an empty L2Logs object with no logs.
   * @returns A new FunctionL2Logs object with no logs.
   */
  public static empty(): UnencryptedFunctionL2Logs {
    return new UnencryptedFunctionL2Logs([]);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns Deserialized instance of `FunctionL2Logs`.
   */
  public static fromBuffer(buf: Buffer, isLengthPrefixed = true): UnencryptedFunctionL2Logs {
    const reader = new BufferReader(buf, 0);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const logs = reader.readBufferArray(logsBufLength);

    return new UnencryptedFunctionL2Logs(logs.map(UnencryptedL2Log.fromBuffer));
  }

  /**
   * Creates a new L2Logs object with `numLogs` logs.
   * @param numLogs - The number of logs to create.
   * @returns A new UnencryptedFunctionL2Logs object.
   */
  public static random(numLogs: number): UnencryptedFunctionL2Logs {
    if (numLogs > MAX_UNENCRYPTED_LOGS_PER_CALL) {
      throw new Error(`Trying to create ${numLogs} logs for one call (max: ${MAX_UNENCRYPTED_LOGS_PER_CALL})`);
    }
    const logs: UnencryptedL2Log[] = [];
    for (let i = 0; i < numLogs; i++) {
      logs.push(UnencryptedL2Log.random());
    }
    return new UnencryptedFunctionL2Logs(logs);
  }

  /**
   * Convert a plain JSON object to a FunctionL2Logs class object.
   * @param obj - A plain FunctionL2Logs JSON object.
   * @returns A FunctionL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const logs = obj.logs.map(UnencryptedL2Log.fromJSON);
    return new UnencryptedFunctionL2Logs(logs);
  }
}
