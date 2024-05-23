import { AztecAddress, Fr, FunctionData, FunctionSelector, TxContext, TxRequest, Vector } from '@aztec/circuits.js';
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
     * Selector of the function to call.
     */
    public functionSelector: FunctionSelector,
    /**
     * The hash of arguments of first call to be executed (usually account entrypoint).
     * @dev This hash is a pointer to `argsOfCalls` unordered array.
     */
    public firstCallArgsHash: Fr,
    /**
     * Transaction context.
     */
    public txContext: TxContext,
    /**
     * An unordered array of packed arguments for each call in the transaction.
     * @dev These arguments are accessed in Noir via oracle and constrained against the args hash. The length of
     * the array is equal to the number of function calls in the transaction (1 args per 1 call).
     */
    public argsOfCalls: PackedValues[],
    /**
     * Transient authorization witnesses for authorizing the execution of one or more actions during this tx.
     * These witnesses are not expected to be stored in the local witnesses database of the PXE.
     */
    public authWitnesses: AuthWitness[],
  ) {}

  toTxRequest(): TxRequest {
    return new TxRequest(
      this.origin,
      // Entrypoints must be private as as defined by the protocol.
      new FunctionData(this.functionSelector, true /* isPrivate */),
      this.firstCallArgsHash,
      this.txContext,
    );
  }

  static getFields(fields: FieldsOf<TxExecutionRequest>) {
    return [
      fields.origin,
      fields.functionSelector,
      fields.firstCallArgsHash,
      fields.txContext,
      fields.argsOfCalls,
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
      this.functionSelector,
      this.firstCallArgsHash,
      this.txContext,
      new Vector(this.argsOfCalls),
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
   * @returns The deserialized TxRequest object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxExecutionRequest {
    const reader = BufferReader.asReader(buffer);
    return new TxExecutionRequest(
      reader.readObject(AztecAddress),
      reader.readObject(FunctionSelector),
      Fr.fromBuffer(reader),
      reader.readObject(TxContext),
      reader.readVector(PackedValues),
      reader.readVector(AuthWitness),
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
