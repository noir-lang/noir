import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';
import { assertItemsLength, assertMemberLength, FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import {
  CONTRACT_TREE_HEIGHT,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  KERNEL_PUBLIC_DATA_READS_LENGTH,
  KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
  L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_DATA_TREE_HEIGHT,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
} from '../constants.js';
import { PreviousKernelData } from '../kernel/previous_kernel_data.js';
import { MembershipWitness } from '../membership_witness.js';
import { UInt32 } from '../shared.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { GlobalVariables } from '../global_variables.js';

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
export class ConstantBaseRollupData {
  constructor(
    /**
     * Snapshot of the private data tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the contract tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the L1-to-L2 message tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricL1ToL2MsgTreeRootsSnapshot: AppendOnlyTreeSnapshot,

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
      reader.readObject(GlobalVariables),
    );
  }

  static getFields(fields: FieldsOf<ConstantBaseRollupData>) {
    return [
      fields.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.startTreeOfHistoricContractTreeRootsSnapshot,
      fields.startTreeOfHistoricL1ToL2MsgTreeRootsSnapshot,
      fields.privateKernelVkTreeRoot,
      fields.publicKernelVkTreeRoot,
      fields.baseRollupVkHash,
      fields.mergeRollupVkHash,
      fields.globalVariables,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...ConstantBaseRollupData.getFields(this));
  }
}

/**
 * Inputs to the base rollup circuit.
 */
export class BaseRollupInputs {
  /**
   * Height of the private data subtree which is to be inserted into the private data tree.
   * Note: There are notes from 2 kernels being processed here so kenrnel new commitments length is multiplied by 2.
   */
  public static PRIVATE_DATA_SUBTREE_HEIGHT = Math.log2(KERNEL_NEW_COMMITMENTS_LENGTH * 2);
  /**
   * Height of the contract subtree which is to be inserted into the contract tree.
   */
  public static CONTRACT_SUBTREE_HEIGHT = Math.log2(KERNEL_NEW_CONTRACTS_LENGTH * 2);
  /**
   * Height of the nullifier subtree which is to be inserted into the nullifier tree.
   */
  public static NULLIFIER_SUBTREE_HEIGHT = Math.log2(KERNEL_NEW_NULLIFIERS_LENGTH * 2);

  constructor(
    /**
     * Data of the 2 kernels that preceded this base rollup circuit.
     */
    public kernelData: [PreviousKernelData, PreviousKernelData],

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
     * The nullifiers which need to be udpated to perform the batch insertion of the new nullifiers.
     * See `StandardIndexedTree.batchInsert` function for more details.
     */
    public lowNullifierLeafPreimages: NullifierLeafPreimage[],
    /**
     * Membership witnesses for the nullifiers which need to be updated to perform the batch insertion of the new
     * nullifiers.
     */
    public lowNullifierMembershipWitness: MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>[],

    /**
     * Sibling path "pointing to" where the new commitments subtree should be inserted into the private data tree.
     */
    public newCommitmentsSubtreeSiblingPath: Fr[],
    /**
     * Sibling path "pointing to" where the new nullifiers subtree should be inserted into the nullifier tree.
     */
    public newNullifiersSubtreeSiblingPath: Fr[],
    /**
     * Sibling path "pointing to" where the new contracts subtree should be inserted into the contract tree.
     */
    public newContractsSubtreeSiblingPath: Fr[],
    /**
     * Sibling paths of leaves which are to be affected by the public data update requests.
     * Each item in the array is the sibling path that corresponds to an update request.
     */
    public newPublicDataUpdateRequestsSiblingPaths: Fr[][],
    /**
     * Sibling paths of leaves which are to be read by the public data reads.
     * Each item in the array is the sibling path that corresponds to a read request.
     */
    public newPublicDataReadsSiblingPaths: Fr[][],

    /**
     * Membership witnesses of private data tree roots referred by each of the 2 kernels.
     * Can be used to prove that the root is actually in the roots tree.
     */
    public historicPrivateDataTreeRootMembershipWitnesses: [
      MembershipWitness<typeof PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>,
      MembershipWitness<typeof PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>,
    ],
    /**
     * Membership witnesses of contract tree roots referred by each of the 2 kernels.
     */
    public historicContractsTreeRootMembershipWitnesses: [
      MembershipWitness<typeof CONTRACT_TREE_ROOTS_TREE_HEIGHT>,
      MembershipWitness<typeof CONTRACT_TREE_ROOTS_TREE_HEIGHT>,
    ],
    /**
     * Membership witnesses of L1-to-L2 message tree roots referred by each of the 2 kernels.
     */
    public historicL1ToL2MsgTreeRootMembershipWitnesses: [
      MembershipWitness<typeof L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT>,
      MembershipWitness<typeof L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT>,
    ],

    /**
     * Data which is not modified by the base rollup circuit.
     */
    public constants: ConstantBaseRollupData,
  ) {
    assertMemberLength(this, 'lowNullifierLeafPreimages', 2 * KERNEL_NEW_NULLIFIERS_LENGTH);
    assertMemberLength(this, 'lowNullifierMembershipWitness', 2 * KERNEL_NEW_NULLIFIERS_LENGTH);
    assertMemberLength(
      this,
      'newCommitmentsSubtreeSiblingPath',
      PRIVATE_DATA_TREE_HEIGHT - BaseRollupInputs.PRIVATE_DATA_SUBTREE_HEIGHT,
    );
    assertMemberLength(
      this,
      'newNullifiersSubtreeSiblingPath',
      NULLIFIER_TREE_HEIGHT - BaseRollupInputs.NULLIFIER_SUBTREE_HEIGHT,
    );
    assertMemberLength(
      this,
      'newContractsSubtreeSiblingPath',
      CONTRACT_TREE_HEIGHT - BaseRollupInputs.CONTRACT_SUBTREE_HEIGHT,
    );
    assertMemberLength(this, 'newPublicDataUpdateRequestsSiblingPaths', 2 * KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH);
    assertMemberLength(this, 'newPublicDataReadsSiblingPaths', 2 * KERNEL_PUBLIC_DATA_READS_LENGTH);
    assertItemsLength(this, 'newPublicDataUpdateRequestsSiblingPaths', PUBLIC_DATA_TREE_HEIGHT);
    assertItemsLength(this, 'newPublicDataReadsSiblingPaths', PUBLIC_DATA_TREE_HEIGHT);
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
      fields.startPublicDataTreeRoot,
      fields.lowNullifierLeafPreimages,
      fields.lowNullifierMembershipWitness,
      fields.newCommitmentsSubtreeSiblingPath,
      fields.newNullifiersSubtreeSiblingPath,
      fields.newContractsSubtreeSiblingPath,
      fields.newPublicDataUpdateRequestsSiblingPaths,
      fields.newPublicDataReadsSiblingPaths,
      fields.historicPrivateDataTreeRootMembershipWitnesses,
      fields.historicContractsTreeRootMembershipWitnesses,
      fields.historicL1ToL2MsgTreeRootMembershipWitnesses,
      fields.constants,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }
}
