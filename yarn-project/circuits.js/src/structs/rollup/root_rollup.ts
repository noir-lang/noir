import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import {
  ARCHIVE_HEIGHT,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_FIELDS_PER_SHA256,
} from '../../constants.gen.js';
import { AggregationObject } from '../aggregation_object.js';
import { Header } from '../header.js';
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
    public newL1ToL2MessageTreeRootSiblingPath: Tuple<Fr, typeof L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Snapshot of the L1 to L2 message tree at the start of the rollup.
     */
    public startL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the historical block roots tree at the start of the rollup.
     */
    public startArchiveSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Sibling path of the new block tree root.
     */
    public newArchiveSiblingPath: Tuple<Fr, typeof ARCHIVE_HEIGHT>,
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
      fields.newL1ToL2MessageTreeRootSiblingPath,
      fields.startL1ToL2MessageTreeSnapshot,
      fields.startArchiveSnapshot,
      fields.newArchiveSiblingPath,
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
    /** Native aggregation state at the end of the rollup. */
    public aggregationObject: AggregationObject,
    /** Snapshot of archive tree after this block/rollup been processed */
    public archive: AppendOnlyTreeSnapshot,
    /** A header of an L2 block. */
    public header: Header,
    /** Hash of the L1 to L2 messages. */
    public l1ToL2MessagesHash: [Fr, Fr],
  ) {}

  static getFields(fields: FieldsOf<RootRollupPublicInputs>) {
    return [fields.aggregationObject, fields.archive, fields.header, fields.l1ToL2MessagesHash] as const;
  }

  toBuffer() {
    return serializeToBuffer(...RootRollupPublicInputs.getFields(this));
  }

  static from(fields: FieldsOf<RootRollupPublicInputs>): RootRollupPublicInputs {
    return new RootRollupPublicInputs(...RootRollupPublicInputs.getFields(fields));
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
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(Header),
      reader.readArray(NUM_FIELDS_PER_SHA256, Fr) as [Fr, Fr],
    );
  }
}
