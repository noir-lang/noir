import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { Proof } from '../proof.js';
import { ParityPublicInputs } from './parity_public_inputs.js';

export class RootParityInput {
  constructor(
    /** The proof of the execution of the parity circuit. */
    public readonly proof: Proof,
    /** The public inputs of the parity circuit. */
    public readonly publicInputs: ParityPublicInputs,
  ) {}

  toBuffer() {
    return serializeToBuffer(...RootParityInput.getFields(this));
  }

  static from(fields: FieldsOf<RootParityInput>): RootParityInput {
    return new RootParityInput(...RootParityInput.getFields(fields));
  }

  static getFields(fields: FieldsOf<RootParityInput>) {
    return [fields.proof, fields.publicInputs] as const;
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new RootParityInput(reader.readObject(Proof), reader.readObject(ParityPublicInputs));
  }
}
