import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

export class ParityPublicInputs {
  constructor(
    /** Root of the SHA256 tree. */
    public shaRoot: Fr,
    /** Root of the converted tree. */
    public convertedRoot: Fr,
    /** Root of the VK tree */
    public vkTreeRoot: Fr,
  ) {
    if (shaRoot.toBuffer()[0] != 0) {
      throw new Error(`shaRoot buffer must be 31 bytes. Got 32 bytes`);
    }
  }

  /**
   * Serializes the inputs to a buffer.
   * @returns The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(...ParityPublicInputs.getFields(this));
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The inputs serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Creates a new ParityPublicInputs instance from the given fields.
   * @param fields - The fields to create the instance from.
   * @returns The instance.
   */
  static from(fields: FieldsOf<ParityPublicInputs>): ParityPublicInputs {
    return new ParityPublicInputs(...ParityPublicInputs.getFields(fields));
  }

  /**
   * Extracts the fields from the given instance.
   * @param fields - The instance to get the fields from.
   * @returns The instance fields.
   */
  static getFields(fields: FieldsOf<ParityPublicInputs>) {
    return [fields.shaRoot, fields.convertedRoot, fields.vkTreeRoot] as const;
  }

  /**
   * Deserializes the inputs from a buffer.
   * @param buffer - The buffer to deserialize from.
   * @returns A new ParityPublicInputs instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ParityPublicInputs(reader.readObject(Fr), reader.readObject(Fr), Fr.fromBuffer(reader));
  }

  /**
   * Deserializes the inputs from a hex string.
   * @param str - The hex string to deserialize from.
   * @returns A new ParityPublicInputs instance.
   */
  static fromString(str: string) {
    return ParityPublicInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}
