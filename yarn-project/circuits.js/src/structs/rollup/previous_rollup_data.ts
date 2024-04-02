import { serializeToBuffer } from '@aztec/foundation/serialize';

import { type ROLLUP_VK_TREE_HEIGHT } from '../../constants.gen.js';
import { type MembershipWitness } from '../membership_witness.js';
import { type Proof } from '../proof.js';
import { type UInt32 } from '../shared.js';
import { type VerificationKey } from '../verification_key.js';
import { type BaseOrMergeRollupPublicInputs } from './base_or_merge_rollup_public_inputs.js';

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
    public proof: Proof,
    /**
     * The verification key of the base or merge rollup circuit.
     */
    public vk: VerificationKey,
    /**
     * The index of the rollup circuit's vk in a big tree of rollup circuit vks.
     */
    public vkIndex: UInt32,
    /**
     * Sibling path of the rollup circuit's vk in a big tree of rollup circuit vks.
     */
    public vkSiblingPath: MembershipWitness<typeof ROLLUP_VK_TREE_HEIGHT>,
  ) {}

  /**
   * Serializes previous rollup data to a buffer.
   * @returns The buffer of the serialized previous rollup data.
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.baseOrMergeRollupPublicInputs, this.proof, this.vk, this.vkIndex, this.vkSiblingPath);
  }
}
