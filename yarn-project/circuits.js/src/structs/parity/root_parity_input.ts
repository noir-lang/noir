import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { RecursiveProof } from '../recursive_proof.js';
import { VerificationKeyAsFields } from '../verification_key.js';
import { ParityPublicInputs } from './parity_public_inputs.js';

export class RootParityInput<PROOF_LENGTH extends number> {
  constructor(
    /** The proof of the execution of the parity circuit. */
    public readonly proof: RecursiveProof<PROOF_LENGTH>,
    /** The circuit's verification key */
    public readonly verificationKey: VerificationKeyAsFields,
    /** The public inputs of the parity circuit. */
    public readonly publicInputs: ParityPublicInputs,
  ) {}

  toBuffer() {
    return serializeToBuffer(...RootParityInput.getFields(this));
  }

  static from<PROOF_LENGTH extends number>(
    fields: FieldsOf<RootParityInput<PROOF_LENGTH>>,
  ): RootParityInput<PROOF_LENGTH> {
    return new RootParityInput(...RootParityInput.getFields(fields));
  }

  static getFields<PROOF_LENGTH extends number>(fields: FieldsOf<RootParityInput<PROOF_LENGTH>>) {
    return [fields.proof, fields.verificationKey, fields.publicInputs] as const;
  }

  static fromBuffer<PROOF_LENGTH extends number>(buffer: Buffer | BufferReader, size: PROOF_LENGTH) {
    const reader = BufferReader.asReader(buffer);
    return new RootParityInput(
      RecursiveProof.fromBuffer<PROOF_LENGTH>(reader, size),
      reader.readObject(VerificationKeyAsFields),
      reader.readObject(ParityPublicInputs),
    );
  }
}
