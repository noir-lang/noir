import { BufferReader, serializeBufferToVector } from '@aztec/foundation/serialize';
import { randomBytes } from 'crypto';
import { sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/circuits.js';

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
    const serializedLogs = this.logs.map(buffer => serializeBufferToVector(buffer));
    return serializeBufferToVector(Buffer.concat(serializedLogs));
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
  public hash(): [Fr, Fr] {
    const hash = sha256(this.toBuffer());

    // TS version of https://github.com/AztecProtocol/aztec-packages/blob/e2e3bf1dbeda5199060fb1711200d20414557cd4/circuits/cpp/src/aztec3/circuits/hash.hpp#L330
    // Split the hash into two fields, a high and a low
    const buf1 = Buffer.concat([Buffer.alloc(16), hash.subarray(0, 16)]);
    const buf2 = Buffer.concat([Buffer.alloc(16), hash.subarray(16, 32)]);

    return [Fr.fromBuffer(buf1), Fr.fromBuffer(buf2)];
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns Deserialized instance of `FunctionL2Logs`.
   */
  public static fromBuffer(buf: Buffer, isLengthPrefixed = true): FunctionL2Logs {
    const offset = isLengthPrefixed ? 4 : 0;
    const reader = new BufferReader(buf, offset);

    const logs = reader.readBufferArray();
    return new FunctionL2Logs(logs);
  }

  /**
   * Creates a new L2Logs object with `numLogs` logs.
   * @param numLogs - The number of logs to create.
   * @returns A new FunctionL2Logs object.
   */
  public static random(numLogs: number): FunctionL2Logs {
    const logs: Buffer[] = [];
    for (let i = 0; i < numLogs; i++) {
      logs.push(randomBytes(144));
    }
    return new FunctionL2Logs(logs);
  }
}
