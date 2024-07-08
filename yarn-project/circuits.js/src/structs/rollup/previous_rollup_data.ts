import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { NESTED_RECURSIVE_PROOF_LENGTH, VK_TREE_HEIGHT } from '../../constants.gen.js';
import { MembershipWitness } from '../membership_witness.js';
import { RecursiveProof } from '../recursive_proof.js';
import { VerificationKeyAsFields } from '../verification_key.js';
import { BaseOrMergeRollupPublicInputs } from './base_or_merge_rollup_public_inputs.js';

/**
 * Represents the data of a previous merge or base rollup circuit.
 */
export class PreviousRollupData {
  constructor(
    /**
     * Public inputs to the base or merge rollup circuit.
     */
    public baseOrMergeRollupPublicInputs: BaseOrMergeRollupPublicInputs,
    /**
     * The proof of the base or merge rollup circuit.
     */
    public proof: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
    /**
     * The verification key of the base or merge rollup circuit.
     */
    public vk: VerificationKeyAsFields,
    /**
     * Sibling path of the rollup circuit's vk in a big tree of rollup circuit vks.
     */
    public vkWitness: MembershipWitness<typeof VK_TREE_HEIGHT>,
  ) {}

  /**
   * Serializes previous rollup data to a buffer.
   * @returns The buffer of the serialized previous rollup data.
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.baseOrMergeRollupPublicInputs, this.proof, this.vk, this.vkWitness);
  }

  /**
   * Deserializes previous rollup data from a buffer.
   * @param buffer - A buffer to deserialize from.
   * @returns A new PreviousRollupData instance.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): PreviousRollupData {
    const reader = BufferReader.asReader(buffer);
    return new PreviousRollupData(
      reader.readObject(BaseOrMergeRollupPublicInputs),
      RecursiveProof.fromBuffer(reader, NESTED_RECURSIVE_PROOF_LENGTH),
      reader.readObject(VerificationKeyAsFields),
      MembershipWitness.fromBuffer(reader, VK_TREE_HEIGHT),
    );
  }
}
