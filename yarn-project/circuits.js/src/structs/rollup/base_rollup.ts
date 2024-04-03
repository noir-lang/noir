import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import {
  type ARCHIVE_HEIGHT,
  type MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  type PUBLIC_DATA_TREE_HEIGHT,
} from '../../constants.gen.js';
import { GlobalVariables } from '../global_variables.js';
import { type KernelData } from '../kernel/kernel_data.js';
import { type MembershipWitness } from '../membership_witness.js';
import { type PartialStateReference } from '../partial_state_reference.js';
import { type UInt32 } from '../shared.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { NullifierLeaf, NullifierLeafPreimage } from './nullifier_leaf/index.js';
import { PublicDataTreeLeaf, PublicDataTreeLeafPreimage } from './public_data_leaf/index.js';
import { type StateDiffHints } from './state_diff_hints.js';

export { NullifierLeaf, NullifierLeafPreimage, PublicDataTreeLeaf, PublicDataTreeLeafPreimage };

/**
 * Data which is forwarded through the base rollup circuits unchanged.
 */
export class ConstantRollupData {
  constructor(
    /** Archive tree snapshot at the very beginning of the entire rollup. */
    public lastArchive: AppendOnlyTreeSnapshot,

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
      fields.lastArchive,
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
    /** Data of the 2 kernels that preceded this base rollup circuit. */
    public kernelData: KernelData,
    /** Partial state reference at the start of the rollup. */
    public start: PartialStateReference,
    /** Hints used while proving state diff validity. */
    public stateDiffHints: StateDiffHints,

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
      fields.start,
      fields.stateDiffHints,
      fields.sortedPublicDataWrites,
      fields.sortedPublicDataWritesIndexes,
      fields.lowPublicDataWritesPreimages,
      fields.lowPublicDataWritesMembershipWitnesses,
      fields.archiveRootMembershipWitness,
      fields.constants,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }
}
