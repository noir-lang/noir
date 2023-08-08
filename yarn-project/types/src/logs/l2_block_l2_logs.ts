import { BufferReader, serializeBufferToVector } from '@aztec/foundation/serialize';

import { TxL2Logs } from './tx_l2_logs.js';

/**
 * Data container of logs emitted in all txs in a given L2 block.
 */
export class L2BlockL2Logs {
  constructor(
    /**
     * An array containing logs emitted in individual function invocations in this tx.
     */
    public readonly txLogs: TxL2Logs[],
  ) {}

  /**
   * Serializes logs into a buffer.
   * @returns A buffer containing the serialized logs.
   */
  public toBuffer(): Buffer {
    const serializedTxLogs = this.txLogs.map(logs => logs.toBuffer());
    // Concatenate all serialized function logs into a single buffer and prefix it with 4 bytes for its total length.
    return serializeBufferToVector(Buffer.concat(serializedTxLogs));
  }

  /**
   * Get the total length of serialized data.
   * @returns Total length of serialized data.
   */
  public getSerializedLength(): number {
    return this.txLogs.reduce((acc, logs) => acc + logs.getSerializedLength(), 0) + 4;
  }

  /**
   * Deserializes logs from a buffer.
   * @param buffer - The buffer containing the serialized logs.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): L2BlockL2Logs {
    const reader = BufferReader.asReader(buffer);

    const logsBufLength = reader.readNumber();
    const serializedTxLogs = reader.readBufferArray(logsBufLength);

    const txLogs = serializedTxLogs.map(logs => TxL2Logs.fromBuffer(logs, false));
    return new L2BlockL2Logs(txLogs);
  }

  /**
   * Creates a new `L2BlockL2Logs` object with `numFunctionInvocations` function logs and `numLogsIn1Invocation` logs
   * in each invocation.
   * @param numTxs - The number of txs in the block.
   * @param numFunctionInvocations - The number of function invocations in the tx.
   * @param numLogsIn1Invocation - The number of logs emitted in each function invocation.
   * @returns A new `L2BlockL2Logs` object.
   */
  public static random(numTxs: number, numFunctionInvocations: number, numLogsIn1Invocation: number): L2BlockL2Logs {
    const txLogs: TxL2Logs[] = [];
    for (let i = 0; i < numTxs; i++) {
      txLogs.push(TxL2Logs.random(numFunctionInvocations, numLogsIn1Invocation));
    }
    return new L2BlockL2Logs(txLogs);
  }

  /**
   * Unrolls logs from a set of blocks.
   * @param blockLogs - Input logs from a set of blocks.
   * @returns Unrolled logs.
   */
  public static unrollLogs(blockLogs: L2BlockL2Logs[]): Buffer[] {
    const logs: Buffer[] = [];
    for (const blockLog of blockLogs) {
      for (const txLog of blockLog.txLogs) {
        for (const functionLog of txLog.functionLogs) {
          logs.push(...functionLog.logs);
        }
      }
    }
    return logs;
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
   * Convert a plain JSON object to a L2BlockL2Logs class object.
   * @param obj - A plain L2BlockL2Logs JSON object.
   * @returns A L2BlockL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const txLogs = obj.txLogs.map((log: any) => TxL2Logs.fromJSON(log));
    return new L2BlockL2Logs(txLogs);
  }
}
