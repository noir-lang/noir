import { sha256 } from '@aztec/foundation/crypto';
import { BufferReader, prefixBufferWithLength, truncateAndPad } from '@aztec/foundation/serialize';

import { EncryptedL2Log } from './encrypted_l2_log.js';
import { UnencryptedL2Log } from './unencrypted_l2_log.js';

/**
 * Data container of logs emitted in 1 function invocation (corresponds to 1 kernel iteration).
 */
export abstract class FunctionL2Logs<TLog extends UnencryptedL2Log | EncryptedL2Log> {
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
    // Adding 4 to each log's length to account for the size stored in the serialized buffer and then one more time
    // adding 4 for the resulting buffer length.
    return this.logs.reduce((acc, log) => acc + log.length + 4, 0) + 4;
  }

  /**
   * Calculates hash of serialized logs.
   * @returns 2 fields containing all 256 bits of information of sha256 hash.
   */
  public hash(): Buffer {
    // Remove first 4 bytes that are occupied by length which is not part of the preimage in contracts and L2Blocks
    const preimage = this.toBuffer().subarray(4);
    return truncateAndPad(sha256(preimage));
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
   * @param logType - The type of logs to generate.
   * @returns A new EncryptedFunctionL2Logs object.
   */
  public static random(numLogs: number): EncryptedFunctionL2Logs {
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
   * @param logType - The type of logs to generate.
   * @returns A new UnencryptedFunctionL2Logs object.
   */
  public static random(numLogs: number): UnencryptedFunctionL2Logs {
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
