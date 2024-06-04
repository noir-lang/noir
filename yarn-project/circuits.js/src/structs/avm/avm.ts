import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { PublicCircuitPublicInputs } from '../public_circuit_public_inputs.js';
import { Vector } from '../shared.js';

export class AvmHint {
  constructor(public readonly key: Fr, public readonly value: Fr) {}

  /**
   * Serializes the inputs to a buffer.
   * @returns - The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(...AvmHint.getFields(this));
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The instance serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Is the struct empty?
   * @returns whether all members are empty.
   */
  isEmpty(): boolean {
    return this.key.isEmpty() && this.value.isEmpty();
  }

  /**
   * Creates a new instance from fields.
   * @param fields - Fields to create the instance from.
   * @returns A new AvmHint instance.
   */
  static from(fields: FieldsOf<AvmHint>): AvmHint {
    return new AvmHint(...AvmHint.getFields(fields));
  }

  /**
   * Extracts fields from an instance.
   * @param fields - Fields to create the instance from.
   * @returns An array of fields.
   */
  static getFields(fields: FieldsOf<AvmHint>) {
    return [fields.key, fields.value] as const;
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buff: Buffer | BufferReader): AvmHint {
    const reader = BufferReader.asReader(buff);
    return new AvmHint(Fr.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  /**
   * Deserializes from a hex string.
   * @param str - Hex string to read from.
   * @returns The deserialized instance.
   */
  static fromString(str: string): AvmHint {
    return AvmHint.fromBuffer(Buffer.from(str, 'hex'));
  }
}

export class AvmExecutionHints {
  constructor(
    public readonly storageValues: Vector<AvmHint>,
    public readonly noteHashExists: Vector<AvmHint>,
    public readonly nullifierExists: Vector<AvmHint>,
    public readonly l1ToL2MessageExists: Vector<AvmHint>,
  ) {}

  /**
   * Serializes the inputs to a buffer.
   * @returns - The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(...AvmExecutionHints.getFields(this));
  }

  toBufferVM() {
    return serializeToBuffer(this.flat());
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The instance serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Is the struct empty?
   * @returns whether all members are empty.
   */
  isEmpty(): boolean {
    return (
      this.storageValues.items.length == 0 &&
      this.noteHashExists.items.length == 0 &&
      this.nullifierExists.items.length == 0 &&
      this.l1ToL2MessageExists.items.length == 0
    );
  }

  /**
   * Creates a new instance from fields.
   * @param fields - Fields to create the instance from.
   * @returns A new AvmExecutionHints instance.
   */
  static from(fields: FieldsOf<AvmExecutionHints>): AvmExecutionHints {
    return new AvmExecutionHints(...AvmExecutionHints.getFields(fields));
  }

  /**
   * Extracts fields from an instance.
   * @param fields - Fields to create the instance from.
   * @returns An array of fields.
   */
  static getFields(fields: FieldsOf<AvmExecutionHints>) {
    return [fields.storageValues, fields.noteHashExists, fields.nullifierExists, fields.l1ToL2MessageExists] as const;
  }

  flat() {
    return [
      ...this.storageValues.items,
      ...this.noteHashExists.items,
      ...this.nullifierExists.items,
      ...this.l1ToL2MessageExists.items,
    ];
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buff: Buffer | BufferReader): AvmExecutionHints {
    const reader = BufferReader.asReader(buff);
    return new AvmExecutionHints(
      new Vector(reader.readVector(AvmHint)),
      new Vector(reader.readVector(AvmHint)),
      new Vector(reader.readVector(AvmHint)),
      new Vector(reader.readVector(AvmHint)),
    );
  }

  /**
   * Deserializes from a hex string.
   * @param str - Hex string to read from.
   * @returns The deserialized instance.
   */
  static fromString(str: string): AvmCircuitInputs {
    return AvmCircuitInputs.fromBuffer(Buffer.from(str, 'hex'));
  }

  /**
   * Construct an empty instance.
   * @returns The empty instance.
   */
  static empty() {
    return new AvmExecutionHints(new Vector([]), new Vector([]), new Vector([]), new Vector([]));
  }
}

export class AvmCircuitInputs {
  constructor(
    public readonly bytecode: Buffer,
    public readonly calldata: Fr[],
    public readonly publicInputs: PublicCircuitPublicInputs,
    public readonly avmHints: AvmExecutionHints,
  ) {}

  /**
   * Serializes the inputs to a buffer.
   * @returns - The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.bytecode.length,
      this.bytecode,
      this.calldata.length,
      this.calldata,
      this.publicInputs.toBuffer(),
      this.avmHints.toBuffer(),
    );
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The instance serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Is the struct empty?
   * @returns whether all members are empty.
   */
  isEmpty(): boolean {
    return (
      this.bytecode.length == 0 && this.calldata.length == 0 && this.publicInputs.isEmpty() && this.avmHints.isEmpty()
    );
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
    return [fields.bytecode, fields.calldata, fields.publicInputs, fields.avmHints] as const;
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buff: Buffer | BufferReader): AvmCircuitInputs {
    const reader = BufferReader.asReader(buff);
    return new AvmCircuitInputs(
      reader.readBuffer(),
      reader.readVector(Fr),
      PublicCircuitPublicInputs.fromBuffer(reader),
      AvmExecutionHints.fromBuffer(reader),
    );
  }

  /**
   * Deserializes from a hex string.
   * @param str - Hex string to read from.
   * @returns The deserialized instance.
   */
  static fromString(str: string): AvmCircuitInputs {
    return AvmCircuitInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}
