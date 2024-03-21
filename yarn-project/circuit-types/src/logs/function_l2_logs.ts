import { randomBytes, sha256 } from '@aztec/foundation/crypto';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, prefixBufferWithLength, truncateAndPad } from '@aztec/foundation/serialize';

import { LogType } from './log_type.js';
import { UnencryptedL2Log } from './unencrypted_l2_log.js';

/**
 * Data container of logs emitted in 1 function invocation (corresponds to 1 kernel iteration).
 */
export class FunctionL2Logs {
  constructor(
    /**
     * An array of logs.
     */
    public readonly logs: Buffer[],
  ) {}

  /**
   * Serializes all function logs into a buffer.
   * @returns A buffer containing the serialized logs.
   * @remarks Each log is prefixed with 4 bytes for its length, then all the serialized logs are concatenated and
   *          the resulting buffer is prefixed with 4 bytes for its total length.
   */
  public toBuffer(): Buffer {
    const serializedLogs = this.logs.map(buffer => prefixBufferWithLength(buffer));
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
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns Deserialized instance of `FunctionL2Logs`.
   */
  public static fromBuffer(buf: Buffer, isLengthPrefixed = true): FunctionL2Logs {
    const reader = new BufferReader(buf, 0);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const logs = reader.readBufferArray(logsBufLength);

    return new FunctionL2Logs(logs);
  }

  /**
   * Creates a new L2Logs object with `numLogs` logs.
   * @param numLogs - The number of logs to create.
   * @param logType - The type of logs to generate.
   * @returns A new FunctionL2Logs object.
   */
  public static random(numLogs: number, logType = LogType.ENCRYPTED): FunctionL2Logs {
    const logs: Buffer[] = [];
    for (let i = 0; i < numLogs; i++) {
      if (logType === LogType.ENCRYPTED) {
        const randomEphPubKey = Point.random();
        const randomLogContent = randomBytes(144 - Point.SIZE_IN_BYTES);
        logs.push(Buffer.concat([Fr.random().toBuffer(), randomLogContent, randomEphPubKey.toBuffer()]));
      } else {
        logs.push(UnencryptedL2Log.random().toBuffer());
      }
    }
    return new FunctionL2Logs(logs);
  }

  /**
   * Creates an empty L2Logs object with no logs.
   * @returns A new FunctionL2Logs object with no logs.
   */
  public static empty(): FunctionL2Logs {
    return new FunctionL2Logs([]);
  }

  /**
   * Convert a FunctionL2Logs class object to a plain JSON object.
   * @returns A plain object with FunctionL2Logs properties.
   */
  public toJSON() {
    return {
      logs: this.logs.map(log => log.toString('hex')),
    };
  }

  /**
   * Convert a plain JSON object to a FunctionL2Logs class object.
   * @param obj - A plain FunctionL2Logs JSON object.
   * @returns A FunctionL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const logs = obj.logs.map((log: string) => Buffer.from(log, 'hex'));
    return new FunctionL2Logs(logs);
  }
}
