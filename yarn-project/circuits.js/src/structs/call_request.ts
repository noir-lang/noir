import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

/**
 * Caller context.
 */
export class CallerContext {
  constructor(
    /**
     * Address of the caller contract.
     */
    public msgSender: AztecAddress,
    /**
     * Storage contract address of the caller contract.
     */
    public storageContractAddress: AztecAddress,
  ) {}

  /**
   * Returns a new instance of CallerContext with zero values.
   * @returns A new instance of CallerContext with zero values.
   */
  public static empty(): CallerContext {
    return new CallerContext(AztecAddress.ZERO, AztecAddress.ZERO);
  }

  isEmpty() {
    return this.msgSender.isZero() && this.storageContractAddress.isZero();
  }

  static from(fields: FieldsOf<CallerContext>): CallerContext {
    return new CallerContext(...CallerContext.getFields(fields));
  }

  static getFields(fields: FieldsOf<CallerContext>) {
    return [fields.msgSender, fields.storageContractAddress] as const;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(...CallerContext.getFields(this));
  }

  /**
   * Deserialize this from a buffer.
   * @param buffer - The bufferable type from which to deserialize.
   * @returns The deserialized instance of PublicCallRequest.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new CallerContext(new AztecAddress(reader.readBytes(32)), new AztecAddress(reader.readBytes(32)));
  }

  equals(callerContext: CallerContext) {
    return (
      callerContext.msgSender.equals(this.msgSender) &&
      callerContext.storageContractAddress.equals(this.storageContractAddress)
    );
  }
}

/**
 * Call request.
 */
export class CallRequest {
  constructor(
    /**
     * The hash of the call stack item.
     */
    public hash: Fr,
    /**
     * The address of the contract calling the function.
     */
    public callerContractAddress: AztecAddress,
    /**
     * The call context of the contract calling the function.
     */
    public callerContext: CallerContext,
    /**
     * The call context of the contract calling the function.
     */
    public startSideEffectCounter: Fr,
    /**
     * The call context of the contract calling the function.
     */
    public endSideEffectCounter: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.hash,
      this.callerContractAddress,
      this.callerContext,
      this.startSideEffectCounter,
      this.endSideEffectCounter,
    );
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance of CallRequest.
   */
  public static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new CallRequest(
      Fr.fromBuffer(reader),
      reader.readObject(AztecAddress),
      reader.readObject(CallerContext),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  isEmpty() {
    return (
      this.hash.isZero() &&
      this.callerContractAddress.isZero() &&
      this.callerContext.isEmpty() &&
      this.startSideEffectCounter.isZero() &&
      this.endSideEffectCounter.isZero()
    );
  }

  /**
   * Returns a new instance of CallRequest with zero hash, caller contract address and caller context.
   * @returns A new instance of CallRequest with zero hash, caller contract address and caller context.
   */
  public static empty() {
    return new CallRequest(Fr.ZERO, AztecAddress.ZERO, CallerContext.empty(), Fr.ZERO, Fr.ZERO);
  }

  equals(callRequest: CallRequest) {
    return (
      callRequest.hash.equals(this.hash) &&
      callRequest.callerContractAddress.equals(this.callerContractAddress) &&
      callRequest.callerContext.equals(this.callerContext) &&
      callRequest.startSideEffectCounter.equals(this.startSideEffectCounter) &&
      callRequest.endSideEffectCounter.equals(this.endSideEffectCounter)
    );
  }

  toString() {
    return `CallRequest(hash: ${this.hash.toString()}, callerContractAddress: ${this.callerContractAddress.toString()}, callerContext: ${this.callerContext.toString()}, startSideEffectCounter: ${this.startSideEffectCounter.toString()}, endSideEffectCounter: ${this.endSideEffectCounter.toString()})`;
  }
}
