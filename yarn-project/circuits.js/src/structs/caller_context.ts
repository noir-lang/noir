import { AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

export class CallerContext {
  constructor(
    /**
     * Message sender of the caller contract.
     */
    public msgSender: AztecAddress,
    /**
     * Storage contract address of the caller contract.
     */
    public storageContractAddress: AztecAddress,
    /**
     * Whether the caller was modifying state.
     */
    public isStaticCall: boolean,
  ) {}

  toFields(): Fr[] {
    return serializeToFields([this.msgSender, this.storageContractAddress, this.isStaticCall]);
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new CallerContext(reader.readObject(AztecAddress), reader.readObject(AztecAddress), reader.readBoolean());
  }

  /**
   * Returns a new instance of CallerContext with zero values.
   * @returns A new instance of CallerContext with zero values.
   */
  public static empty(): CallerContext {
    return new CallerContext(AztecAddress.ZERO, AztecAddress.ZERO, false);
  }

  isEmpty() {
    return this.msgSender.isZero() && this.storageContractAddress.isZero() && !this.isStaticCall;
  }

  static from(fields: FieldsOf<CallerContext>): CallerContext {
    return new CallerContext(...CallerContext.getFields(fields));
  }

  static getFields(fields: FieldsOf<CallerContext>) {
    return [fields.msgSender, fields.storageContractAddress, fields.isStaticCall] as const;
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
    return new CallerContext(
      new AztecAddress(reader.readBytes(32)),
      new AztecAddress(reader.readBytes(32)),
      reader.readBoolean(),
    );
  }

  equals(callerContext: CallerContext) {
    return (
      callerContext.msgSender.equals(this.msgSender) &&
      callerContext.storageContractAddress.equals(this.storageContractAddress) &&
      callerContext.isStaticCall === this.isStaticCall
    );
  }
}
