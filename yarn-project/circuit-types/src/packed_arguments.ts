import { Fr, Vector } from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

/**
 * Packs a set of arguments into a hash.
 */
export class PackedArguments {
  constructor(
    /**
     *  Function arguments.
     */
    public args: Fr[],
    /**
     * The hash of the args
     */
    public hash: Fr,
  ) {}

  static getFields(fields: FieldsOf<PackedArguments>) {
    return [fields.args, fields.hash] as const;
  }

  static from(fields: FieldsOf<PackedArguments>): PackedArguments {
    return new PackedArguments(...PackedArguments.getFields(fields));
  }

  static fromArgs(args: Fr[]) {
    return new PackedArguments(args, computeVarArgsHash(args));
  }

  toBuffer() {
    return serializeToBuffer(new Vector(this.args), this.hash);
  }

  static fromBuffer(buffer: Buffer | BufferReader): PackedArguments {
    const reader = BufferReader.asReader(buffer);
    return new PackedArguments(reader.readVector(Fr), Fr.fromBuffer(reader));
  }
}
