import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { VK_TREE_HEIGHT } from '../../constants.gen.js';
import { RecursiveProof } from '../recursive_proof.js';
import { VerificationKeyAsFields } from '../verification_key.js';
import { ParityPublicInputs } from './parity_public_inputs.js';

export class RootParityInput<PROOF_LENGTH extends number> {
  constructor(
    /** The proof of the execution of the parity circuit. */
    public readonly proof: RecursiveProof<PROOF_LENGTH>,
    /** The circuit's verification key */
    public readonly verificationKey: VerificationKeyAsFields,
    /** The vk path in the vk tree*/
    public readonly vkPath: Tuple<Fr, typeof VK_TREE_HEIGHT>,
    /** The public inputs of the parity circuit. */
    public readonly publicInputs: ParityPublicInputs,
  ) {}

  toBuffer() {
    return serializeToBuffer(...RootParityInput.getFields(this));
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  static from<PROOF_LENGTH extends number>(
    fields: FieldsOf<RootParityInput<PROOF_LENGTH>>,
  ): RootParityInput<PROOF_LENGTH> {
    return new RootParityInput(...RootParityInput.getFields(fields));
  }

  static getFields<PROOF_LENGTH extends number>(fields: FieldsOf<RootParityInput<PROOF_LENGTH>>) {
    return [fields.proof, fields.verificationKey, fields.vkPath, fields.publicInputs] as const;
  }

  static fromBuffer<PROOF_LENGTH extends number | undefined>(
    buffer: Buffer | BufferReader,
    expectedSize?: PROOF_LENGTH,
  ): RootParityInput<PROOF_LENGTH extends number ? PROOF_LENGTH : number> {
    const reader = BufferReader.asReader(buffer);
    return new RootParityInput(
      RecursiveProof.fromBuffer<PROOF_LENGTH>(reader, expectedSize),
      reader.readObject(VerificationKeyAsFields),
      reader.readArray(VK_TREE_HEIGHT, Fr),
      reader.readObject(ParityPublicInputs),
    );
  }

  static fromString<PROOF_LENGTH extends number | undefined>(
    str: string,
    expectedSize?: PROOF_LENGTH,
  ): RootParityInput<PROOF_LENGTH extends number ? PROOF_LENGTH : number> {
    return RootParityInput.fromBuffer(Buffer.from(str, 'hex'), expectedSize);
  }
}
