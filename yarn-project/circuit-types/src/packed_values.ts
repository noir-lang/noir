import { Fr, Vector } from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**
 * Packs a set of values into a hash.
 */
export class PackedValues {
  private constructor(
    /**
     *  Raw values.
     */
    public readonly values: Fr[],
    /**
     * The hash of the raw values
     */
    public readonly hash: Fr,
  ) {}

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
