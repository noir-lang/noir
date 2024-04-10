import { Fr, Vector } from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

/**
 * Packs a set of values into a hash.
 */
export class PackedValues {
  constructor(
    /**
     *  Raw values.
     */
    public values: Fr[],
    /**
     * The hash of the raw values
     */
    public hash: Fr,
  ) {}

  static getFields(fields: FieldsOf<PackedValues>) {
    return [fields.values, fields.hash] as const;
  }

  static from(fields: FieldsOf<PackedValues>): PackedValues {
    return new PackedValues(...PackedValues.getFields(fields));
  }

  static fromValues(values: Fr[]) {
    return new PackedValues(values, computeVarArgsHash(values));
  }

  toBuffer() {
    return serializeToBuffer(new Vector(this.values), this.hash);
  }

  static fromBuffer(buffer: Buffer | BufferReader): PackedValues {
    const reader = BufferReader.asReader(buffer);
    return new PackedValues(reader.readVector(Fr), Fr.fromBuffer(reader));
  }
}
