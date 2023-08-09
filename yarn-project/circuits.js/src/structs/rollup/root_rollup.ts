import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import {
  HISTORIC_BLOCKS_TREE_HEIGHT,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
} from '../../cbind/constants.gen.js';
import { FieldsOf, assertMemberLength } from '../../utils/jsUtils.js';
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
    public newL1ToL2Messages: Fr[],
    /**
     * Sibling path of the new L1 to L2 message tree root.
     */
    public newL1ToL2MessageTreeRootSiblingPath: Fr[],
    /**
     * Snapshot of the L1 to L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historic block roots tree at the start of the rollup.
     */
    public startHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Sibling path of the new historic block roots tree root.
     */
    public newHistoricBlocksTreeSiblingPath: Fr[],
  ) {
    assertMemberLength(this, 'newL1ToL2MessageTreeRootSiblingPath', L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH);
    assertMemberLength(this, 'newHistoricBlocksTreeSiblingPath', HISTORIC_BLOCKS_TREE_HEIGHT);
    assertMemberLength(this, 'newL1ToL2Messages', NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);
  }

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
      fields.newL1ToL2MessageTreeRootSiblingPath,
      fields.startL1ToL2MessageTreeSnapshot,
      fields.startHistoricBlocksTreeSnapshot,
      fields.newHistoricBlocksTreeSiblingPath,
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
     * Snapshot of the private data tree at the start of the rollup.
     */
    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the private data tree at the end of the rollup.
     */
    public endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,

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
     * Snapshot of the historic private data tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historic private data tree roots tree at the end of the rollup.
     */
    public endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the historic contract tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historic contract tree roots tree at the end of the rollup.
     */
    public endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the L1 to L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the L1 to L2 message tree at the end of the rollup.
     */
    public endL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the historic L1 to L2 message tree roots tree at the start of the rollup.
     */
    public startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historic L1 to L2 message tree roots tree at the end of the rollup.
     */
    public endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the historic blocks tree roots tree at the start of the rollup.
     */
    public startHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historic blocks tree roots tree at the end of the rollup.
     */
    public endHistoricBlocksTreeSnapshot: AppendOnlyTreeSnapshot,

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
      fields.startPrivateDataTreeSnapshot,
      fields.endPrivateDataTreeSnapshot,
      fields.startNullifierTreeSnapshot,
      fields.endNullifierTreeSnapshot,
      fields.startContractTreeSnapshot,
      fields.endContractTreeSnapshot,
      fields.startPublicDataTreeRoot,
      fields.endPublicDataTreeRoot,
      fields.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.startTreeOfHistoricContractTreeRootsSnapshot,
      fields.endTreeOfHistoricContractTreeRootsSnapshot,
      fields.startL1ToL2MessageTreeSnapshot,
      fields.endL1ToL2MessageTreeSnapshot,
      fields.startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      fields.endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      fields.startHistoricBlocksTreeSnapshot,
      fields.endHistoricBlocksTreeSnapshot,
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
      reader.readFr(),
      reader.readFr(),
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
      [reader.readFr(), reader.readFr()],
      [reader.readFr(), reader.readFr()],
    );
  }
}
