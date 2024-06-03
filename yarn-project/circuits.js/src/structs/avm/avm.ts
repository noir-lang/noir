import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/6774): add public inputs.
export class AvmCircuitInputs {
  constructor(public readonly bytecode: Buffer, public readonly calldata: Fr[]) {}

  /**
   * Serializes the inputs to a buffer.
   * @returns - The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.bytecode.length, this.bytecode, this.calldata.length, this.calldata);
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The instance serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Creates a new instance from fields.
   * @param fields - Fields to create the instance from.
   * @returns A new AvmCircuitInputs instance.
   */
  static from(fields: FieldsOf<AvmCircuitInputs>): AvmCircuitInputs {
    return new AvmCircuitInputs(...AvmCircuitInputs.getFields(fields));
  }

  /**
   * Extracts fields from an instance.
   * @param fields - Fields to create the instance from.
   * @returns An array of fields.
   */
  static getFields(fields: FieldsOf<AvmCircuitInputs>) {
    return [fields.bytecode, fields.calldata] as const;
  }

  static fromBuffer(buff: Buffer | BufferReader): AvmCircuitInputs {
    const reader = BufferReader.asReader(buff);
    return new AvmCircuitInputs(reader.readBuffer(), reader.readVector(Fr));
  }

  static fromString(str: string): AvmCircuitInputs {
    return AvmCircuitInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}
