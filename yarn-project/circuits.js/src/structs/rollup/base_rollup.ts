import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple } from '@aztec/foundation/serialize';

import {
  ARCHIVE_HEIGHT,
  CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
} from '../../constants.gen.js';
import { FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { GlobalVariables } from '../global_variables.js';
import { PreviousKernelData } from '../kernel/previous_kernel_data.js';
import { MembershipWitness } from '../membership_witness.js';
import { UInt32 } from '../shared.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { NullifierLeaf, NullifierLeafPreimage } from './nullifier_leaf/index.js';
import { PublicDataTreeLeaf, PublicDataTreeLeafPreimage } from './public_data_leaf/index.js';

export { NullifierLeaf, NullifierLeafPreimage, PublicDataTreeLeaf, PublicDataTreeLeafPreimage };

/**
 * Data which is forwarded through the base rollup circuits unchanged.
 */
export class ConstantRollupData {
  constructor(
    /**
     * Snapshot of the blocks tree at the start of the rollup.
     */
    public archiveSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Root of the private kernel verification key tree.
     */
    public privateKernelVkTreeRoot: Fr,
    /**
     * Root of the public kernel circuit verification key tree.
     */
    public publicKernelVkTreeRoot: Fr,
    /**
     * Hash of the base rollup circuit verification key.
     */
    public baseRollupVkHash: Fr,
    /**
     * Hash of the merge rollup circuit verification key.
     */
    public mergeRollupVkHash: Fr,
    /**
     * Global variables for the block
     */
    public globalVariables: GlobalVariables,
  ) {}

  static from(fields: FieldsOf<ConstantRollupData>): ConstantRollupData {
    return new ConstantRollupData(...ConstantRollupData.getFields(fields));
  }

  static fromBuffer(buffer: Buffer | BufferReader): ConstantRollupData {
    const reader = BufferReader.asReader(buffer);
    return new ConstantRollupData(
      reader.readObject(AppendOnlyTreeSnapshot),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readObject(GlobalVariables),
    );
  }

  static getFields(fields: FieldsOf<ConstantRollupData>) {
    return [
      fields.archiveSnapshot,
      fields.privateKernelVkTreeRoot,
      fields.publicKernelVkTreeRoot,
      fields.baseRollupVkHash,
      fields.mergeRollupVkHash,
      fields.globalVariables,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...ConstantRollupData.getFields(this));
  }
}

/**
 * Inputs to the base rollup circuit.
 */
export class BaseRollupInputs {
  constructor(
    /**
     * Data of the 2 kernels that preceded this base rollup circuit.
     */
    public kernelData: PreviousKernelData,
    /**
     * Snapshot of the note hash tree at the start of the base rollup circuit.
     */
    public startNoteHashTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the nullifier tree at the start of the base rollup circuit.
     */
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the contract tree at the start of the base rollup circuit.
     */
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the public data tree at the start of the base rollup circuit.
     */
    public startPublicDataTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the blocks tree at the start of the base rollup circuit.
     */
    public archiveSnapshot: AppendOnlyTreeSnapshot,

    /**
     * The nullifiers to be inserted in the tree, sorted high to low.
     */
    public sortedNewNullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The indexes of the sorted nullifiers to the original ones.
     */
    public sortednewNullifiersIndexes: Tuple<UInt32, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The nullifiers which need to be updated to perform the batch insertion of the new nullifiers.
     * See `StandardIndexedTree.batchInsert` function for more details.
     */
    public lowNullifierLeafPreimages: Tuple<NullifierLeafPreimage, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Membership witnesses for the nullifiers which need to be updated to perform the batch insertion of the new
     * nullifiers.
     */
    public lowNullifierMembershipWitness: Tuple<
      MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>,
      typeof MAX_NEW_NULLIFIERS_PER_TX
    >,

    /**
     * Sibling path "pointing to" where the new commitments subtree should be inserted into the note hash tree.
     */
    public newCommitmentsSubtreeSiblingPath: Tuple<Fr, typeof NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Sibling path "pointing to" where the new nullifiers subtree should be inserted into the nullifier tree.
     */
    public newNullifiersSubtreeSiblingPath: Tuple<Fr, typeof NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Sibling path "pointing to" where the new contracts subtree should be inserted into the contract tree.
     */
    public newContractsSubtreeSiblingPath: Tuple<Fr, typeof CONTRACT_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * The public data writes to be inserted in the tree, sorted high slot to low slot.
     */
    public sortedPublicDataWrites: Tuple<PublicDataTreeLeaf, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,

    /**
     * The indexes of the sorted public data writes to the original ones.
     */
    public sortedPublicDataWritesIndexes: Tuple<UInt32, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * The public data writes which need to be updated to perform the batch insertion of the new public data writes.
     * See `StandardIndexedTree.batchInsert` function for more details.
     */
    public lowPublicDataWritesPreimages: Tuple<
      PublicDataTreeLeafPreimage,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
    /**
     * Membership witnesses for the nullifiers which need to be updated to perform the batch insertion of the new
     * nullifiers.
     */
    public lowPublicDataWritesMembershipWitnesses: Tuple<
      MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,

    /**
     * Sibling path "pointing to" where the new public data subtree should be inserted into the public data tree.
     */
    public publicDataWritesSubtreeSiblingPath: Tuple<Fr, typeof PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH>,

    /**
     * Preimages of leaves which are to be read by the public data reads.
     */
    public publicDataReadsPreimages: Tuple<PublicDataTreeLeafPreimage, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
    /**
     * Sibling paths of leaves which are to be read by the public data reads.
     * Each item in the array is the sibling path that corresponds to a read request.
     */
    public publicDataReadsMembershipWitnesses: Tuple<
      MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_PUBLIC_DATA_READS_PER_TX
    >,

    /**
     * Membership witnesses of blocks referred by each of the 2 kernels.
     */
    public archiveRootMembershipWitness: MembershipWitness<typeof ARCHIVE_HEIGHT>,
    /**
     * Data which is not modified by the base rollup circuit.
     */
    public constants: ConstantRollupData,
  ) {}

  static from(fields: FieldsOf<BaseRollupInputs>): BaseRollupInputs {
    return new BaseRollupInputs(...BaseRollupInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<BaseRollupInputs>) {
    return [
      fields.kernelData,
      fields.startNoteHashTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.startPublicDataTreeSnapshot,
      fields.archiveSnapshot,
      fields.sortedNewNullifiers,
      fields.sortednewNullifiersIndexes,
      fields.lowNullifierLeafPreimages,
      fields.lowNullifierMembershipWitness,
      fields.newCommitmentsSubtreeSiblingPath,
      fields.newNullifiersSubtreeSiblingPath,
      fields.newContractsSubtreeSiblingPath,
      fields.sortedPublicDataWrites,
      fields.sortedPublicDataWritesIndexes,
      fields.lowPublicDataWritesPreimages,
      fields.lowPublicDataWritesMembershipWitnesses,
      fields.publicDataWritesSubtreeSiblingPath,
      fields.publicDataReadsPreimages,
      fields.publicDataReadsMembershipWitnesses,
      fields.archiveRootMembershipWitness,
      fields.constants,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }
}
