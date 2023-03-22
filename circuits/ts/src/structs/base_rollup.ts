import { assertLength, FieldsOf } from "../utils/jsUtils.js";
import { serializeToBuffer } from "../wasm/serialize.js";
import {
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
} from "./constants.js";
import { PreviousKernelData } from "./kernel.js";
import { AggregationObject, Fr, MembershipWitness, UInt32 } from "./shared.js";

export class NullifierLeafPreimage {
  constructor(
    public leafValue: Fr,
    public nextValue: Fr,
    public nextIndex: UInt32
  ) {}

  toBuffer() {
    return serializeToBuffer(this.leafValue, this.nextValue, this.nextIndex);
  }
}

export class AppendOnlyTreeSnapshot {
  constructor(public root: Fr, public nextAvailableLeafIndex: UInt32) {}

  toBuffer() {
    return serializeToBuffer(this.root, this.nextAvailableLeafIndex);
  }
}

export class ConstantBaseRollupData {
  constructor(
    // The very latest roots as at the very beginning of the entire rollup:
    public startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public treeOfHistoricL1ToL2MsgTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    // Some members of this struct tbd:
    public privateKernelVkTreeRoot: Fr,
    public publicKernelVkTreeRoot: Fr,
    public baseRollupVkHash: Fr,
    public mergeRollupVkHash: Fr
  ) {}

  static from(
    fields: FieldsOf<ConstantBaseRollupData>
  ): ConstantBaseRollupData {
    return new ConstantBaseRollupData(
      ...ConstantBaseRollupData.getFields(fields)
    );
  }

  static getFields(fields: FieldsOf<ConstantBaseRollupData>) {
    return [
      fields.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.startTreeOfHistoricContractTreeRootsSnapshot,
      fields.treeOfHistoricL1ToL2MsgTreeRootsSnapshot,
      fields.privateKernelVkTreeRoot,
      fields.publicKernelVkTreeRoot,
      fields.baseRollupVkHash,
      fields.mergeRollupVkHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...ConstantBaseRollupData.getFields(this));
  }
}

/**
 * Inputs to the base rollup circuit
 */
export class BaseRollupInputs {
  constructor(
    public kernelData: [PreviousKernelData, PreviousKernelData],

    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public lowNullifierLeafPreimages: NullifierLeafPreimage[],
    public lowNullifierMembershipWitness: MembershipWitness<
      typeof NULLIFIER_TREE_HEIGHT
    >[],

    public historicPrivateDataTreeRootMembershipWitnesses: [
      MembershipWitness<typeof PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>,
      MembershipWitness<typeof PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>
    ],
    public historicContractsTreeRootMembershipWitnesses: [
      MembershipWitness<typeof CONTRACT_TREE_ROOTS_TREE_HEIGHT>,
      MembershipWitness<typeof CONTRACT_TREE_ROOTS_TREE_HEIGHT>
    ],

    public constants: ConstantBaseRollupData,

    public proverId: Fr
  ) {
    assertLength(
      this,
      "lowNullifierLeafPreimages",
      2 * KERNEL_NEW_NULLIFIERS_LENGTH
    );
    assertLength(
      this,
      "lowNullifierMembershipWitness",
      2 * KERNEL_NEW_NULLIFIERS_LENGTH
    );
  }

  static from(fields: FieldsOf<BaseRollupInputs>): BaseRollupInputs {
    return new BaseRollupInputs(...BaseRollupInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<BaseRollupInputs>) {
    return [
      fields.kernelData,
      fields.startNullifierTreeSnapshot,
      fields.lowNullifierLeafPreimages,
      fields.lowNullifierMembershipWitness,
      fields.historicPrivateDataTreeRootMembershipWitnesses,
      fields.historicContractsTreeRootMembershipWitnesses,
      fields.constants,
      fields.proverId,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }
}

enum RollupTypes {
  Base = 0,
  Rollup = 1,
}

/**
 * Output of the base rollup circuit
 */
export class BaseRollupPublicInputs {
  constructor(
    public rollupType: RollupTypes.Base,

    public endAggregationObject: AggregationObject,
    public constants: ConstantBaseRollupData,

    // The only tree root actually updated in this circuit is the nullifier tree, because earlier leaves (of low_nullifiers) must be updated to point to the new nullifiers in this circuit.
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public endNullifierTreeSnapshots: AppendOnlyTreeSnapshot,

    public newCommitmentsSubtreeRoot: Fr,
    public newNullifiersSubtreeRoot: Fr,
    public newContractLeavesSubtreeRoot: Fr,

    // Hashes (probably sha256) to make public inputs constant-sized (to then be unpacked on-chain)
    public newCommitmentsHash: Fr,
    public newNullifiersHash: Fr,
    public newL1MsgsHash: Fr,
    public newContractDataHash: Fr,
    public proverContributionsHash: Fr
  ) {}

  static fromBuffer(buffer: Buffer): BaseRollupPublicInputs {
    throw new Error("Not implemented");
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.rollupType.valueOf(), // TODO: Check the size of the enum in cpp land
      this.endAggregationObject,
      this.constants,

      this.startNullifierTreeSnapshot,
      this.endNullifierTreeSnapshots,

      this.newCommitmentsSubtreeRoot,
      this.newNullifiersSubtreeRoot,
      this.newContractLeavesSubtreeRoot,

      this.newCommitmentsHash,
      this.newNullifiersHash,
      this.newL1MsgsHash,
      this.newContractDataHash,
      this.proverContributionsHash
    );
  }
}
