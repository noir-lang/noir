import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple } from '@aztec/foundation/serialize';

import {
  BLOCKS_TREE_HEIGHT,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
} from '../../constants.gen.js';
import { FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { AggregationObject } from '../aggregation_object.js';
import { GlobalVariables } from '../global_variables.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { PreviousRollupData } from './previous_rollup_data.js';

/**
 * Represents inputs of the root rollup circuit.
 */
export class RootRollupInputs {
  constructor(
    /**
     * The previous rollup data.
     * Note: Root rollup circuit is the latest circuit the chain of circuits and the previous rollup data is the data
     * from 2 merge or base rollup circuits.
     */
    public previousRollupData: [PreviousRollupData, PreviousRollupData],
    /**
     * New L1 to L2 messages.
     */
    public newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
    /**
     * Sibling path of the new L1 to L2 message tree root.
     */
    public newL1ToL2MessagesTreeRootSiblingPath: Tuple<Fr, typeof L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Snapshot of the L1 to L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historical block roots tree at the start of the rollup.
     */
    public startBlocksTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Sibling path of the new block tree root.
     */
    public newBlocksTreeSiblingPath: Tuple<Fr, typeof BLOCKS_TREE_HEIGHT>,
  ) {}

  toBuffer() {
    return serializeToBuffer(...RootRollupInputs.getFields(this));
  }

  static from(fields: FieldsOf<RootRollupInputs>): RootRollupInputs {
    return new RootRollupInputs(...RootRollupInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<RootRollupInputs>) {
    return [
      fields.previousRollupData,
      fields.newL1ToL2Messages,
      fields.newL1ToL2MessagesTreeRootSiblingPath,
      fields.startL1ToL2MessagesTreeSnapshot,
      fields.startBlocksTreeSnapshot,
      fields.newBlocksTreeSiblingPath,
    ] as const;
  }
}

/**
 * Represents public inputs of the root rollup circuit.
 *
 * NOTE: in practice, we'll hash all of this up into a single public input, for cheap on-chain verification.
 */
export class RootRollupPublicInputs {
  constructor(
    /**
     * Native aggregation state at the end of the rollup.
     */
    public endAggregationObject: AggregationObject,

    /**
     * Global variables of the L2 block.
     */
    public globalVariables: GlobalVariables,
    /**
     * Snapshot of the note hash tree at the start of the rollup.
     */
    public startNoteHashTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the note hash tree at the end of the rollup.
     */
    public endNoteHashTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the nullifier tree at the start of the rollup.
     */
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the nullifier tree at the end of the rollup.
     */
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the contract tree at the start of the rollup.
     */
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the contract tree at the end of the rollup.
     */
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Root of the public data tree at the start of the rollup.
     */
    public startPublicDataTreeRoot: Fr,
    /**
     * Root of the public data tree at the end of the rollup.
     */
    public endPublicDataTreeRoot: Fr,

    /**
     * Snapshot of the historical note hash tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricalNoteHashTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historical note hash tree roots tree at the end of the rollup.
     */
    public endTreeOfHistoricalNoteHashTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the historical contract tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricalContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historical contract tree roots tree at the end of the rollup.
     */
    public endTreeOfHistoricalContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the L1 to L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the L1 to L2 message tree at the end of the rollup.
     */
    public endL1ToL2MessagesTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the historical L1 to L2 message tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricalL1ToL2MessagesTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historical L1 to L2 message tree roots tree at the end of the rollup.
     */
    public endTreeOfHistoricalL1ToL2MessagesTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the blocks tree roots tree at the start of the rollup.
     */
    public startBlocksTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the blocks tree roots tree at the end of the rollup.
     */
    public endBlocksTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Hash of the calldata.
     */
    public calldataHash: [Fr, Fr],
    /**
     * Hash of the L1 to L2 messages.
     */
    public l1ToL2MessagesHash: [Fr, Fr],
  ) {}

  static getFields(fields: FieldsOf<RootRollupPublicInputs>) {
    return [
      fields.endAggregationObject,
      fields.globalVariables,
      fields.startNoteHashTreeSnapshot,
      fields.endNoteHashTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.endNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.endContractTreeSnapshot,
      fields.startPublicDataTreeRoot,
      fields.endPublicDataTreeRoot,
      fields.startTreeOfHistoricalNoteHashTreeRootsSnapshot,
      fields.endTreeOfHistoricalNoteHashTreeRootsSnapshot,
      fields.startTreeOfHistoricalContractTreeRootsSnapshot,
      fields.endTreeOfHistoricalContractTreeRootsSnapshot,
      fields.startL1ToL2MessagesTreeSnapshot,
      fields.endL1ToL2MessagesTreeSnapshot,
      fields.startTreeOfHistoricalL1ToL2MessagesTreeRootsSnapshot,
      fields.endTreeOfHistoricalL1ToL2MessagesTreeRootsSnapshot,
      fields.startBlocksTreeSnapshot,
      fields.endBlocksTreeSnapshot,
      fields.calldataHash,
      fields.l1ToL2MessagesHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...RootRollupPublicInputs.getFields(this));
  }

  static from(fields: FieldsOf<RootRollupPublicInputs>): RootRollupPublicInputs {
    return new RootRollupPublicInputs(...RootRollupPublicInputs.getFields(fields));
  }

  /**
   * Returns the sha256 hash of the calldata.
   * @returns The sha256 hash of the calldata.
   */
  public sha256CalldataHash(): Buffer {
    const high = this.calldataHash[0].toBuffer();
    const low = this.calldataHash[1].toBuffer();

    const hash = Buffer.alloc(32);
    for (let i = 0; i < 16; i++) {
      hash[i] = high[i + 16];
      hash[i + 16] = low[i + 16];
    }

    return hash;
  }

  /**
   * Deserializes a buffer into a `RootRollupPublicInputs` object.
   * @param buffer - The buffer to deserialize.
   * @returns The deserialized `RootRollupPublicInputs` object.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): RootRollupPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new RootRollupPublicInputs(
      reader.readObject(AggregationObject),
      reader.readObject(GlobalVariables),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      [Fr.fromBuffer(reader), Fr.fromBuffer(reader)],
      [Fr.fromBuffer(reader), Fr.fromBuffer(reader)],
    );
  }
}
