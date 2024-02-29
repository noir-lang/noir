import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

export class ReadRequest {
  constructor(
    /**
     * The value being read.
     */
    public value: Fr,
    /**
     * The side-effect counter.
     */
    public counter: number,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of ReadRequest.
   */
  static fromBuffer(buffer: Buffer | BufferReader): ReadRequest {
    const reader = BufferReader.asReader(buffer);
    return new ReadRequest(Fr.fromBuffer(reader), reader.readNumber());
  }

  /**
   * Convert to an array of fields.
   * @returns The array of fields.
   */
  toFields(): Fr[] {
    return [this.value, new Fr(this.counter)];
  }

  static fromFields(fields: Fr[] | FieldReader): ReadRequest {
    const reader = FieldReader.asReader(fields);
    return new ReadRequest(reader.readField(), reader.readU32());
  }

  /**
   * Returns whether this instance of side-effect is empty.
   * @returns True if the value and counter both are zero.
   */
  isEmpty() {
    return this.value.isZero() && !this.counter;
  }

  /**
   * Returns an empty instance of side-effect.
   * @returns Side-effect with both value and counter being zero.
   */
  static empty(): ReadRequest {
    return new ReadRequest(Fr.zero(), 0);
  }
}

/**
 * ReadRequest with context of the contract emitting the request.
 */
export class ReadRequestContext {
  constructor(
    /**
     * The value being read.
     */
    public value: Fr,
    /**
     * The counter.
     */
    public counter: number,
    /**
     * The address of the contract emitting the request.
     */
    public contractAddress: AztecAddress,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.value, this.counter, this.contractAddress);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of ReadRequestContext.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ReadRequestContext(Fr.fromBuffer(reader), reader.readNumber(), AztecAddress.fromBuffer(reader));
  }

  /**
   * Convert to an array of fields.
   * @returns The array of fields.
   */
  toFields(): Fr[] {
    return [this.value, new Fr(this.counter), this.contractAddress.toField()];
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new ReadRequestContext(reader.readField(), reader.readU32(), AztecAddress.fromField(reader.readField()));
  }

  /**
   * Returns whether this instance of side-effect is empty.
   * @returns True if the value, note hash and counter are all zero.
   */
  isEmpty() {
    return this.value.isZero() && !this.counter && this.contractAddress.isZero();
  }

  /**
   * Returns an empty instance of side-effect.
   * @returns Side-effect with value, note hash and counter being zero.
   */
  static empty(): ReadRequestContext {
    return new ReadRequestContext(Fr.zero(), 0, AztecAddress.ZERO);
  }
}
