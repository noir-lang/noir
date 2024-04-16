import { AztecAddress, Fr, FunctionData, GasSettings, TxContext, TxRequest, Vector } from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { AuthWitness } from './auth_witness.js';
import { PackedValues } from './packed_values.js';

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
     * TODO(#3417): Remove this field and replace with a function selector.
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
    public packedArguments: PackedValues[],
    /**
     * Transient authorization witnesses for authorizing the execution of one or more actions during this tx.
     * These witnesses are not expected to be stored in the local witnesses database of the PXE.
     */
    public authWitnesses: AuthWitness[],

    /** Gas choices for this transaction. */
    public gasSettings: GasSettings,
  ) {}

  toTxRequest(): TxRequest {
    return new TxRequest(this.origin, this.functionData, this.argsHash, this.txContext, this.gasSettings);
  }

  static getFields(fields: FieldsOf<TxExecutionRequest>) {
    return [
      fields.origin,
      fields.functionData,
      fields.argsHash,
      fields.txContext,
      fields.packedArguments,
      fields.authWitnesses,
      fields.gasSettings,
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
      this.gasSettings,
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
   * @returns The deserialized TxRequest object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxExecutionRequest {
    const reader = BufferReader.asReader(buffer);
    return new TxExecutionRequest(
      reader.readObject(AztecAddress),
      reader.readObject(FunctionData),
      Fr.fromBuffer(reader),
      reader.readObject(TxContext),
      reader.readVector(PackedValues),
      reader.readVector(AuthWitness),
      reader.readObject(GasSettings),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns The deserialized TxRequest object.
   */
  static fromString(str: string): TxExecutionRequest {
    return TxExecutionRequest.fromBuffer(Buffer.from(str, 'hex'));
  }
}
