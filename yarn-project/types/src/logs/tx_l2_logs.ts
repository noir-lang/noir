import { BufferReader, serializeBufferToVector } from '@aztec/foundation/serialize';

import { FunctionL2Logs } from './function_l2_logs.js';

/**
 * Data container of logs emitted in 1 tx.
 */
export class TxL2Logs {
  constructor(
    /**
     * An array containing logs emitted in individual function invocations in this tx.
     */
    public readonly functionLogs: FunctionL2Logs[],
  ) {}

  /**
   * Serializes logs into a buffer.
   * @returns A buffer containing the serialized logs.
   */
  public toBuffer(): Buffer {
    const serializedFunctionLogs = this.functionLogs.map(logs => logs.toBuffer());
    // Concatenate all serialized function logs into a single buffer and prefix it with 4 bytes for its total length.
    return serializeBufferToVector(Buffer.concat(serializedFunctionLogs));
  }

  /**
   * Get the total length of serialized data.
   * @returns Total length of serialized data.
   */
  public getSerializedLength(): number {
    return this.functionLogs.reduce((acc, logs) => acc + logs.getSerializedLength(), 0) + 4;
  }

  /**
   * Adds function logs to the existing logs.
   * @param functionLogs - The function logs to add
   * @remarks Used by sequencer to append unencrypted logs emitted in public function calls.
   */
  public addFunctionLogs(functionLogs: FunctionL2Logs[]) {
    this.functionLogs.push(...functionLogs);
  }

  /**
   * Deserializes logs from a buffer.
   * @param buf - The buffer containing the serialized logs.
   * @param isLengthPrefixed - Whether the buffer is prefixed with 4 bytes for its total length.
   * @returns A new L2Logs object.
   */
  public static fromBuffer(buf: Buffer | BufferReader, isLengthPrefixed = true): TxL2Logs {
    const reader = BufferReader.asReader(buf);

    // If the buffer is length prefixed use the length to read the array. Otherwise, the entire buffer is consumed.
    const logsBufLength = isLengthPrefixed ? reader.readNumber() : -1;
    const serializedFunctionLogs = reader.readBufferArray(logsBufLength);

    const functionLogs = serializedFunctionLogs.map(logs => FunctionL2Logs.fromBuffer(logs, false));
    return new TxL2Logs(functionLogs);
  }

  /**
   * Creates a new `TxL2Logs` object with `numFunctionInvocations` function logs and `numLogsIn1Invocation` logs
   * in each invocation.
   * @param numFunctionInvocations - The number of function invocations in the tx.
   * @param numLogsIn1Invocation - The number of logs emitted in each function invocation.
   * @returns A new `TxL2Logs` object.
   */
  public static random(numFunctionInvocations: number, numLogsIn1Invocation: number): TxL2Logs {
    const functionLogs: FunctionL2Logs[] = [];
    for (let i = 0; i < numFunctionInvocations; i++) {
      functionLogs.push(FunctionL2Logs.random(numLogsIn1Invocation));
    }
    return new TxL2Logs(functionLogs);
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
   * Convert a plain JSON object to a TxL2Logs class object.
   * @param obj - A plain TxL2Logs JSON object.
   * @returns A TxL2Logs class object.
   */
  public static fromJSON(obj: any) {
    const functionLogs = obj.functionLogs.map((log: any) => FunctionL2Logs.fromJSON(log));
    return new TxL2Logs(functionLogs);
  }
}
