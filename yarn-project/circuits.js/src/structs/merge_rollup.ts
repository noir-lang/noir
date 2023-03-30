import { Fr } from '@aztec/foundation';
import { FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { AppendOnlyTreeSnapshot, BaseRollupPublicInputs, ConstantBaseRollupData } from './base_rollup.js';
import { ROLLUP_VK_TREE_HEIGHT } from './constants.js';
import { AggregationObject, MembershipWitness, RollupTypes, UInt32, UInt8Vector } from './shared.js';
import { VerificationKey } from './verification_key.js';

export class PreviousRollupData {
  constructor(
    public publicInputs: BaseRollupPublicInputs | MergeRollupPublicInputs,
    public proof: UInt8Vector,
    public vk: VerificationKey,
    /**
     * The index of the rollup circuit's vk in a big tree of rollup circuit vks.
     */
    public vkIndex: UInt32,
    public vkSiblingPath: MembershipWitness<typeof ROLLUP_VK_TREE_HEIGHT>,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.publicInputs, this.proof, this.vk, this.vkIndex, this.vkSiblingPath);
  }
}

export class MergeRollupInputs {
  constructor(public previousRollupData: [PreviousRollupData, PreviousRollupData]) {}

  toBuffer() {
    return serializeToBuffer(this.previousRollupData);
  }
}

export class MergeRollupPublicInputs {
  public rollupType: RollupTypes = RollupTypes.Merge;

  constructor(
    /**
     * Tells us how many layers of recursion we've done, to help with subtree insertion in the root rollup circuit.
     */
    public rollupSubtreeHeight: Fr,

    public endAggregationObject: AggregationObject,

    public constants: ConstantBaseRollupData,

    // The only tree root actually updated in this circuit is the nullifier tree, because earlier leaves (of low_nullifiers) must be updated to point to the new nullifiers in this circuit.
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,

    public newCommitmentsSubtreeRoot: Fr,
    public newNullifiersSubtreeRoot: Fr,
    public newContractLeavesSubtreeRoot: Fr,

    // Hashes (probably sha256) to make public inputs constant-sized (to then be unpacked on-chain)
    // UPDATE: we should instead just hash all of the below into a single value. See big diagram of sha256 hashing bottom-right of here.
    // TODO: I've put `fr`, but these hash values' types might need to be two fields if we want all 256-bits, for security purposes.
    public newCommitmentsHash: Fr,
    public newNullifiersHash: Fr,
    public newL1MsgsHash: Fr,
    public newContractDataHash: Fr,
    public proverContributionsHash: Fr,
  ) {}

  static getFields(fields: FieldsOf<MergeRollupPublicInputs>) {
    return [
      fields.rollupSubtreeHeight,
      fields.endAggregationObject,
      fields.constants,
      fields.startNullifierTreeSnapshot,
      fields.endNullifierTreeSnapshot,
      fields.newCommitmentsSubtreeRoot,
      fields.newNullifiersSubtreeRoot,
      fields.newContractLeavesSubtreeRoot,
      fields.newCommitmentsHash,
      fields.newNullifiersHash,
      fields.newL1MsgsHash,
      fields.newContractDataHash,
      fields.proverContributionsHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...MergeRollupPublicInputs.getFields(this));
  }

  static from(fields: FieldsOf<MergeRollupPublicInputs>): MergeRollupPublicInputs {
    return new MergeRollupPublicInputs(...MergeRollupPublicInputs.getFields(fields));
  }
}
