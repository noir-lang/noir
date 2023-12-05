import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple } from '@aztec/foundation/serialize';
import { IndexedTreeLeaf, IndexedTreeLeafPreimage } from '@aztec/foundation/trees';

import {
  ARCHIVE_HEIGHT,
  CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
  KERNELS_PER_BASE_ROLLUP,
  MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP,
  MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP,
  NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
} from '../../constants.gen.js';
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
export class NullifierLeafPreimage implements IndexedTreeLeafPreimage {
  constructor(
    /**
     * Leaf value inside the indexed tree's linked list.
     */
    public nullifier: Fr,
    /**
     * Next value inside the indexed tree's linked list.
     */
    public nextNullifier: Fr,
    /**
     * Index of the next leaf in the indexed tree's linked list.
     */
    public nextIndex: bigint,
  ) {}

  getKey(): bigint {
    return this.nullifier.toBigInt();
  }

  getNextKey(): bigint {
    return this.nextNullifier.toBigInt();
  }

  getNextIndex(): bigint {
    return this.nextIndex;
  }

  asLeaf(): NullifierLeaf {
    return new NullifierLeaf(this.nullifier);
  }

  toBuffer(): Buffer {
    return Buffer.concat(this.toHashInputs());
  }

  toHashInputs(): Buffer[] {
    return [
      Buffer.from(this.nullifier.toBuffer()),
      Buffer.from(toBufferBE(this.nextIndex, 32)),
      Buffer.from(this.nextNullifier.toBuffer()),
    ];
  }

  clone(): NullifierLeafPreimage {
    return new NullifierLeafPreimage(this.nullifier, this.nextNullifier, this.nextIndex);
  }

  static empty(): NullifierLeafPreimage {
    return new NullifierLeafPreimage(Fr.ZERO, Fr.ZERO, 0n);
  }

  static fromBuffer(buf: Buffer): NullifierLeafPreimage {
    const nullifier = Fr.fromBuffer(buf.subarray(0, 32));
    const nextIndex = toBigIntBE(buf.subarray(32, 64));
    const nextNullifier = Fr.fromBuffer(buf.subarray(64, 96));
    return new NullifierLeafPreimage(nullifier, nextNullifier, nextIndex);
  }

  static fromLeaf(leaf: NullifierLeaf, nextKey: bigint, nextIndex: bigint): NullifierLeafPreimage {
    return new NullifierLeafPreimage(leaf.nullifier, new Fr(nextKey), nextIndex);
  }

  static clone(preimage: NullifierLeafPreimage): NullifierLeafPreimage {
    return new NullifierLeafPreimage(preimage.nullifier, preimage.nextNullifier, preimage.nextIndex);
  }
}

/**
 * A nullifier to be inserted in the nullifier tree.
 */
export class NullifierLeaf implements IndexedTreeLeaf {
  constructor(
    /**
     * Nullifier value.
     */
    public nullifier: Fr,
  ) {}

  getKey(): bigint {
    return this.nullifier.toBigInt();
  }

  toBuffer(): Buffer {
    return this.nullifier.toBuffer();
  }

  isEmpty(): boolean {
    return this.nullifier.isZero();
  }

  static buildDummy(key: bigint): NullifierLeaf {
    return new NullifierLeaf(new Fr(key));
  }

  static fromBuffer(buf: Buffer): NullifierLeaf {
    return new NullifierLeaf(Fr.fromBuffer(buf));
  }
}

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
    public kernelData: Tuple<PreviousKernelData, typeof KERNELS_PER_BASE_ROLLUP>,
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
     * Root of the public data tree at the start of the base rollup circuit.
     */
    public startPublicDataTreeRoot: Fr,
    /**
     * Snapshot of the blocks tree at the start of the base rollup circuit.
     */
    public archiveSnapshot: AppendOnlyTreeSnapshot,

    /**
     * The nullifiers to be inserted in the tree, sorted high to low.
     */
    public sortedNewNullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP>,
    /**
     * The indexes of the sorted nullifiers to the original ones.
     */
    public sortednewNullifiersIndexes: Tuple<UInt32, typeof MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP>,
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
     * Membership witnesses of blocks referred by each of the 2 kernels.
     */
    public archiveRootMembershipWitnesses: Tuple<
      MembershipWitness<typeof ARCHIVE_HEIGHT>,
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
      fields.startNoteHashTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.startPublicDataTreeRoot,
      fields.archiveSnapshot,
      fields.sortedNewNullifiers,
      fields.sortednewNullifiersIndexes,
      fields.lowNullifierLeafPreimages,
      fields.lowNullifierMembershipWitness,
      fields.newCommitmentsSubtreeSiblingPath,
      fields.newNullifiersSubtreeSiblingPath,
      fields.newContractsSubtreeSiblingPath,
      fields.newPublicDataUpdateRequestsSiblingPaths,
      fields.newPublicDataReadsSiblingPaths,
      fields.archiveRootMembershipWitnesses,
      fields.constants,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }
}
