import { INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js/constants';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { BufferReader } from '@aztec/foundation/serialize';

/** A globally unique log id. */
export class LogId {
  /**
   * Parses a log id from a string.
   * @param blockNumber - The block number.
   * @param txIndex - The transaction index.
   * @param logIndex - The log index.
   */
  constructor(
    /** The block number the log was emitted in. */
    public readonly blockNumber: number,
    /** The index of a tx in a block the log was emitted in. */
    public readonly txIndex: number,
    /** The index of a log the tx was emitted in. */
    public readonly logIndex: number,
  ) {
    if (!Number.isInteger(blockNumber) || blockNumber < INITIAL_L2_BLOCK_NUM) {
      throw new Error(`Invalid block number: ${blockNumber}`);
    }
    if (!Number.isInteger(txIndex)) {
      throw new Error(`Invalid tx index: ${txIndex}`);
    }
    if (!Number.isInteger(logIndex)) {
      throw new Error(`Invalid log index: ${logIndex}`);
    }
  }

  /**
   * Serializes log id to a buffer.
   * @returns A buffer containing the serialized log id.
   */
  public toBuffer(): Buffer {
    return Buffer.concat([
      toBufferBE(BigInt(this.blockNumber), 4),
      toBufferBE(BigInt(this.txIndex), 4),
      toBufferBE(BigInt(this.logIndex), 4),
    ]);
  }

  /**
   * Creates a LogId from a buffer.
   * @param buffer - A buffer containing the serialized log id.
   * @returns A log id.
   */
  static fromBuffer(buffer: Buffer | BufferReader): LogId {
    const reader = BufferReader.asReader(buffer);

    const blockNumber = reader.readNumber();
    const txIndex = reader.readNumber();
    const logIndex = reader.readNumber();

    return new LogId(blockNumber, txIndex, logIndex);
  }

  /**
   * Converts the LogId instance to a string.
   * @returns A string representation of the log id.
   */
  public toString(): string {
    return `${this.blockNumber}-${this.txIndex}-${this.logIndex}`;
  }

  /**
   * Creates a LogId from a string.
   * @param data - A string representation of a log id.
   * @returns A log id.
   */
  static fromString(data: string): LogId {
    const [rawBlockNumber, rawTxIndex, rawLogIndex] = data.split('-');
    const blockNumber = Number(rawBlockNumber);
    const txIndex = Number(rawTxIndex);
    const logIndex = Number(rawLogIndex);

    return new LogId(blockNumber, txIndex, logIndex);
  }

  /**
   * Serializes log id to a human readable string.
   * @returns A human readable representation of the log id.
   */
  public toHumanReadable(): string {
    return `logId: (blockNumber: ${this.blockNumber}, txIndex: ${this.txIndex}, logIndex: ${this.logIndex})`;
  }
}
