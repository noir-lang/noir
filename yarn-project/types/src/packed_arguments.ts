import { CircuitsWasm, FieldsOf, Fr, Vector } from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/abis';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';

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

  static async fromArgs(args: Fr[], wasm?: CircuitsWasm) {
    return new PackedArguments(args, await computeVarArgsHash(wasm ?? (await CircuitsWasm.get()), args));
  }

  toBuffer() {
    return serializeToBuffer(new Vector(this.args), this.hash);
  }

  static fromBuffer(buffer: Buffer | BufferReader): PackedArguments {
    const reader = BufferReader.asReader(buffer);
    return new PackedArguments(reader.readVector(Fr), reader.readFr());
  }
}
