import {
  AztecAddress,
  CircuitsWasm,
  EcdsaSignature,
  FieldsOf,
  Fr,
  FunctionData,
  SignedTxRequest,
  TxContext,
  TxRequest,
  Vector,
} from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/abis';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';
import cloneDeep from 'lodash.clonedeep';

/**
 * Request to execute a transaction. Similar to TxRequest, but has the full args.
 */
export class TxExecutionRequest {
  constructor(
    /**
     * Sender.
     */
    public from: AztecAddress,
    /**
     * Target.
     */
    public to: AztecAddress,
    /**
     * Function data representing the function to call.
     */
    public functionData: FunctionData,
    /**
     * Function arguments.
     */
    public args: Fr[],
    /**
     * Tx nonce.
     */
    public nonce: Fr,
    /**
     * Transaction context.
     */
    public txContext: TxContext,
    /**
     * Chain ID of the transaction. Here for replay protection.
     */
    public chainId: Fr,
  ) {}

  // TODO(#663): The only reason why we need to manually create a tx request from a tx execution request
  // is because of direct public function invocations. For private runs, the args hash should be calculated by
  // the private execution simulator, and used to populate the tx request, instead of being manually calculated.
  // This should be removed once we kill direct public function calls when we go full AA.
  async toTxRequest(): Promise<TxRequest> {
    return this.toTxRequestUsingArgsHash(await computeVarArgsHash(await CircuitsWasm.get(), this.args));
  }

  toTxRequestUsingArgsHash(argsHash: Fr): TxRequest {
    return new TxRequest(this.from, this.to, this.functionData, argsHash, this.nonce, this.txContext, this.chainId);
  }

  static getFields(fields: FieldsOf<TxExecutionRequest>) {
    return [
      fields.from,
      fields.to,
      fields.functionData,
      fields.args,
      fields.nonce,
      fields.txContext,
      fields.chainId,
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
      this.from,
      this.to,
      this.functionData,
      new Vector(this.args),
      this.nonce,
      this.txContext,
      this.chainId,
    );
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
      reader.readObject(AztecAddress),
      reader.readObject(FunctionData),
      reader.readVector(Fr),
      reader.readFr(),
      reader.readObject(TxContext),
      reader.readFr(),
    );
  }
}

/**
 * Wraps a TxExecutionRequest with an ECDSA signature.
 */
export class SignedTxExecutionRequest {
  constructor(
    /**
     * Transaction request.
     */
    public txRequest: TxExecutionRequest,
    /**
     * Signature.
     */
    public signature: EcdsaSignature,
  ) {}

  async toSignedTxRequest(): Promise<SignedTxRequest> {
    return new SignedTxRequest(await this.txRequest.toTxRequest(), this.signature);
  }

  clone(): SignedTxExecutionRequest {
    return cloneDeep(this);
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.txRequest, this.signature);
  }

  /**
   * Deserialises from a buffer.
   * @param buffer - The buffer representation of the object.
   * @returns The new object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): SignedTxExecutionRequest {
    const reader = BufferReader.asReader(buffer);
    return new SignedTxExecutionRequest(reader.readObject(TxExecutionRequest), reader.readObject(EcdsaSignature));
  }
}
