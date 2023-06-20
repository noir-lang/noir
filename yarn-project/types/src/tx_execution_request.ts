import { AztecAddress, FieldsOf, Fr, FunctionData, TxContext, TxRequest, Vector } from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';

/**
 * Request to execute a transaction. Similar to TxRequest, but has the full args.
 */
export class TxExecutionRequest {
  constructor(
    /**
     * Sender.
     */
    public origin: AztecAddress,
    /**
     * Function data representing the function to call.
     */
    public functionData: FunctionData,
    /**
     * Function arguments.
     */
    public args: Fr[],
    /**
     * Transaction context.
     */
    public txContext: TxContext,
  ) {}

  toTxRequest(argsHash: Fr): TxRequest {
    return new TxRequest(this.origin, this.functionData, argsHash, this.txContext);
  }

  static getFields(fields: FieldsOf<TxExecutionRequest>) {
    return [fields.origin, fields.functionData, fields.args, fields.txContext] as const;
  }

  static from(fields: FieldsOf<TxExecutionRequest>): TxExecutionRequest {
    return new TxExecutionRequest(...TxExecutionRequest.getFields(fields));
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.origin, this.functionData, new Vector(this.args), this.txContext);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   * @returns The deserialised TxRequest object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxExecutionRequest {
    const reader = BufferReader.asReader(buffer);
    return new TxExecutionRequest(
      reader.readObject(AztecAddress),
      reader.readObject(FunctionData),
      reader.readVector(Fr),
      reader.readObject(TxContext),
    );
  }
}
