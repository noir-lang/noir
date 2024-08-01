import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

export class OptionalNumber {
  constructor(
    /**
     * Whether a value was set.
     */
    public isSome: boolean,
    /**
     * The actual number, if isSome is true.
     */
    public value: number,
  ) {}

  getSize() {
    return this.toBuffer().length;
  }

  static getFields(fields: FieldsOf<OptionalNumber>) {
    return [fields.isSome, fields.value] as const;
  }

  toBuffer() {
    return serializeToBuffer(...OptionalNumber.getFields(this));
  }

  static fromBuffer(buffer: Buffer | BufferReader): OptionalNumber {
    const reader = BufferReader.asReader(buffer);
    return new OptionalNumber(reader.readBoolean(), reader.readNumber());
  }

  toFields(): Fr[] {
    return serializeToFields(...OptionalNumber.getFields(this));
  }

  static fromFields(fields: Fr[] | FieldReader): OptionalNumber {
    const reader = FieldReader.asReader(fields);
    return new OptionalNumber(reader.readBoolean(), reader.readU32());
  }

  isEmpty(): boolean {
    return !this.isSome && !this.value;
  }

  static empty() {
    return new OptionalNumber(false, 0);
  }
}
