import { AztecAddress, FieldsOf, Fr, FunctionData, TxContext, TxRequest, Vector } from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';

import { AuthWitness } from './auth_witness.js';
import { PackedArguments } from './packed_arguments.js';

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
     * The hash of the entry point arguments.
     */
    public argsHash: Fr,
    /**
     * Transaction context.
     */
    public txContext: TxContext,
    /**
     * These packed arguments will be used during transaction simulation.
     * For example, a call to an account contract might contain as many packed arguments
     * as relayed function calls, and one for the entrypoint.
     */
    public packedArguments: PackedArguments[],
    /**
     * Transient authorization witnesses for authorizing the execution of one or more actions during this tx.
     * These witnesses are not expected to be stored in the local witnesses database of the RPC server.
     */
    public authWitnesses: AuthWitness[],
  ) {}

  toTxRequest(): TxRequest {
    return new TxRequest(this.origin, this.functionData, this.argsHash, this.txContext);
  }

  static getFields(fields: FieldsOf<TxExecutionRequest>) {
    return [
      fields.origin,
      fields.functionData,
      fields.argsHash,
      fields.txContext,
      fields.packedArguments,
      fields.authWitnesses,
    ] as const;
  }

  static from(fields: FieldsOf<TxExecutionRequest>): TxExecutionRequest {
    return new TxExecutionRequest(...TxExecutionRequest.getFields(fields));
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.origin,
      this.functionData,
      this.argsHash,
      this.txContext,
      new Vector(this.packedArguments),
      new Vector(this.authWitnesses),
    );
  }

  /**
   * Serialize as a string.
   * @returns The string.
   */
  toString() {
    return this.toBuffer().toString('hex');
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
      reader.readFr(),
      reader.readObject(TxContext),
      reader.readVector(PackedArguments),
      reader.readVector(AuthWitness),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns The deserialised TxRequest object.
   */
  static fromString(str: string): TxExecutionRequest {
    return TxExecutionRequest.fromBuffer(Buffer.from(str, 'hex'));
  }
}
