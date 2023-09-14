import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple } from '@aztec/foundation/serialize';

import {
  CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
  HISTORIC_BLOCKS_TREE_HEIGHT,
  KERNELS_PER_BASE_ROLLUP,
  MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP,
  MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP,
  NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH,
  PUBLIC_DATA_TREE_HEIGHT,
} from '../../cbind/constants.gen.js';
import { FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { GlobalVariables } from '../global_variables.js';
import { PreviousKernelData } from '../kernel/previous_kernel_data.js';
import { MembershipWitness } from '../membership_witness.js';
import { UInt32 } from '../shared.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';

/**
 * Class containing the data of a preimage of a single leaf in the nullifier tree.
 * Note: It's called preimage because this data gets hashed before being inserted as a node into the `IndexedTree`.
 */
export class NullifierLeafPreimage {
  constructor(
    /**
     * Leaf value inside the indexed tree's linked list.
     */
    public leafValue: Fr,
    /**
     * Next value inside the indexed tree's linked list.
     */
    public nextValue: Fr,
    /**
     * Index of the next leaf in the indexed tree's linked list.
     */
    public nextIndex: UInt32,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.leafValue, this.nextValue, this.nextIndex);
  }

  static empty() {
    return new NullifierLeafPreimage(Fr.ZERO, Fr.ZERO, 0);
  }
}

/**
 * Data which is forwarded through the base rollup circuits unchanged.
 */
export class ConstantRollupData {
  constructor(
    /**
     * Snapshot of the historic blocks roots tree at the start of the rollup.
     */
    public startHistoricBlocksTreeRootsSnapshot: AppendOnlyTreeSnapshot,

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
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readObject(GlobalVariables),
    );
  }

  static getFields(fields: FieldsOf<ConstantRollupData>) {
    return [
      fields.startHistoricBlocksTreeRootsSnapshot,
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
    public kernelData: Tuple<PreviousKernelData, typeof KERNELS_PER_BASE_ROLLUP>,
    /**
     * Snapshot of the private data tree at the start of the base rollup circuit.
     */
    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the nullifier tree at the start of the base rollup circuit.
     */
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the contract tree at the start of the base rollup circuit.
     */
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Root of the public data tree at the start of the base rollup circuit.
     */
    public startPublicDataTreeRoot: Fr,
    /**
     * Snapshot of the historic blocks tree at the start of the base rollup circuit.
     */
    public startHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * The nullifiers which need to be updated to perform the batch insertion of the new nullifiers.
     * See `StandardIndexedTree.batchInsert` function for more details.
     */
    public lowNullifierLeafPreimages: Tuple<NullifierLeafPreimage, typeof MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP>,
    /**
     * Membership witnesses for the nullifiers which need to be updated to perform the batch insertion of the new
     * nullifiers.
     */
    public lowNullifierMembershipWitness: Tuple<
      MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>,
      typeof MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP
    >,
    /**
     * Sibling path "pointing to" where the new commitments subtree should be inserted into the private data tree.
     */
    public newCommitmentsSubtreeSiblingPath: Tuple<Fr, typeof PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Sibling path "pointing to" where the new nullifiers subtree should be inserted into the nullifier tree.
     */
    public newNullifiersSubtreeSiblingPath: Tuple<Fr, typeof NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Sibling path "pointing to" where the new contracts subtree should be inserted into the contract tree.
     */
    public newContractsSubtreeSiblingPath: Tuple<Fr, typeof CONTRACT_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Sibling paths of leaves which are to be affected by the public data update requests.
     * Each item in the array is the sibling path that corresponds to an update request.
     */
    public newPublicDataUpdateRequestsSiblingPaths: Tuple<
      Tuple<Fr, typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP
    >,
    /**
     * Sibling paths of leaves which are to be read by the public data reads.
     * Each item in the array is the sibling path that corresponds to a read request.
     */
    public newPublicDataReadsSiblingPaths: Tuple<
      Tuple<Fr, typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP
    >,
    /**
     * Membership witnesses of historic blocks referred by each of the 2 kernels.
     */
    public historicBlocksTreeRootMembershipWitnesses: Tuple<
      MembershipWitness<typeof HISTORIC_BLOCKS_TREE_HEIGHT>,
      typeof KERNELS_PER_BASE_ROLLUP
    >,
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
      fields.startPrivateDataTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.startPublicDataTreeRoot,
      fields.startHistoricBlocksTreeSnapshot,
      fields.lowNullifierLeafPreimages,
      fields.lowNullifierMembershipWitness,
      fields.newCommitmentsSubtreeSiblingPath,
      fields.newNullifiersSubtreeSiblingPath,
      fields.newContractsSubtreeSiblingPath,
      fields.newPublicDataUpdateRequestsSiblingPaths,
      fields.newPublicDataReadsSiblingPaths,
      fields.historicBlocksTreeRootMembershipWitnesses,
      fields.constants,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }
}
