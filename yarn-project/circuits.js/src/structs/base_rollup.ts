import { BufferReader, Fr } from '@aztec/foundation';
import { assertLength, FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import {
  CONTRACT_TREE_HEIGHT,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_DATA_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
} from './constants.js';
import { PreviousKernelData } from './kernel.js';
import { AggregationObject, MembershipWitness, UInt32 } from './shared.js';

export class NullifierLeafPreimage {
  constructor(public leafValue: Fr, public nextValue: Fr, public nextIndex: UInt32) {}

  toBuffer() {
    return serializeToBuffer(this.leafValue, this.nextValue, this.nextIndex);
  }
}

export class AppendOnlyTreeSnapshot {
  constructor(public root: Fr, public nextAvailableLeafIndex: UInt32) {}

  toBuffer() {
    return serializeToBuffer(this.root, this.nextAvailableLeafIndex);
  }

  static fromBuffer(buffer: Buffer | BufferReader): AppendOnlyTreeSnapshot {
    const reader = BufferReader.asReader(buffer);
    return new AppendOnlyTreeSnapshot(reader.readFr(), reader.readNumber());
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
    public mergeRollupVkHash: Fr,
  ) {}

  static from(fields: FieldsOf<ConstantBaseRollupData>): ConstantBaseRollupData {
    return new ConstantBaseRollupData(...ConstantBaseRollupData.getFields(fields));
  }

  static fromBuffer(buffer: Buffer | BufferReader): ConstantBaseRollupData {
    const reader = BufferReader.asReader(buffer);
    return new ConstantBaseRollupData(
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
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
  public static PRIVATE_DATA_SUBTREE_HEIGHT = Math.log2(KERNEL_NEW_COMMITMENTS_LENGTH * 2);
  public static CONTRACT_SUBTREE_HEIGHT = Math.log2(KERNEL_NEW_CONTRACTS_LENGTH * 2);
  public static NULLIFIER_SUBTREE_HEIGHT = Math.log2(KERNEL_NEW_NULLIFIERS_LENGTH * 2);

  constructor(
    public kernelData: [PreviousKernelData, PreviousKernelData],

    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,

    public lowNullifierLeafPreimages: NullifierLeafPreimage[],
    public lowNullifierMembershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>[],

    public newCommitmentsSubtreeSiblingPath: Fr[],
    public newNullifiersSubtreeSiblingPath: Fr[],
    public newContractsSubtreeSiblingPath: Fr[],

    public historicPrivateDataTreeRootMembershipWitnesses: [
      MembershipWitness<typeof PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>,
      MembershipWitness<typeof PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>,
    ],
    public historicContractsTreeRootMembershipWitnesses: [
      MembershipWitness<typeof CONTRACT_TREE_ROOTS_TREE_HEIGHT>,
      MembershipWitness<typeof CONTRACT_TREE_ROOTS_TREE_HEIGHT>,
    ],

    public constants: ConstantBaseRollupData,
  ) {
    assertLength(this, 'lowNullifierLeafPreimages', 2 * KERNEL_NEW_NULLIFIERS_LENGTH);
    assertLength(this, 'lowNullifierMembershipWitness', 2 * KERNEL_NEW_NULLIFIERS_LENGTH);
    assertLength(
      this,
      'newCommitmentsSubtreeSiblingPath',
      PRIVATE_DATA_TREE_HEIGHT - BaseRollupInputs.PRIVATE_DATA_SUBTREE_HEIGHT,
    );
    assertLength(
      this,
      'newNullifiersSubtreeSiblingPath',
      NULLIFIER_TREE_HEIGHT - BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
    );
    assertLength(
      this,
      'newContractsSubtreeSiblingPath',
      CONTRACT_TREE_HEIGHT - BaseRollupInputs.CONTRACT_SUBTREE_HEIGHT,
    );
  }

  static from(fields: FieldsOf<BaseRollupInputs>): BaseRollupInputs {
    return new BaseRollupInputs(...BaseRollupInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<BaseRollupInputs>) {
    return [
      fields.kernelData,
      fields.startPrivateDataTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.lowNullifierLeafPreimages,
      fields.lowNullifierMembershipWitness,
      fields.newCommitmentsSubtreeSiblingPath,
      fields.newNullifiersSubtreeSiblingPath,
      fields.newContractsSubtreeSiblingPath,
      fields.historicPrivateDataTreeRootMembershipWitnesses,
      fields.historicContractsTreeRootMembershipWitnesses,
      fields.constants,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }
}

/**
 * Output of the base rollup circuit
 */
export class BaseRollupPublicInputs {
  constructor(
    public endAggregationObject: AggregationObject,
    public constants: ConstantBaseRollupData,

    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    public endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,

    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,

    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,

    // Hashes (sha256), to make public inputs constant-sized (to then be unpacked on-chain). Length 2 for high and low
    public calldataHash: [Fr, Fr],
  ) {}

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param bufferReader - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): BaseRollupPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new BaseRollupPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(ConstantBaseRollupData),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readArray(2, Fr) as [Fr, Fr],
    );
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.endAggregationObject,
      this.constants,

      this.startPrivateDataTreeSnapshot,
      this.endPrivateDataTreeSnapshot,

      this.startNullifierTreeSnapshot,
      this.endNullifierTreeSnapshot,

      this.startContractTreeSnapshot,
      this.endContractTreeSnapshot,

      this.calldataHash,
    );
  }
}
