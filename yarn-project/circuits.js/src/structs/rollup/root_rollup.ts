import { assertLength, FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import {
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT,
  L1_TO_L2_MESSAGES_SIBLING_PATH_LENGTH,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
} from '../constants.js';
import { PreviousRollupData } from './previous_rollup_data.js';
import { AggregationObject } from '../aggregation_object.js';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

export class RootRollupInputs {
  constructor(
    public previousRollupData: [PreviousRollupData, PreviousRollupData],

    public newHistoricPrivateDataTreeRootSiblingPath: Fr[],
    public newHistoricContractDataTreeRootSiblingPath: Fr[],
    public newL1ToL2Messages: Fr[],
    public newL1ToL2MessageTreeRootSiblingPath: Fr[],
    public newHistoricL1ToL2MessageTreeRootSiblingPath: Fr[],
    public startL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,
    public startHistoricTreeL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot,
  ) {
    assertLength(this, 'newHistoricPrivateDataTreeRootSiblingPath', PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);
    assertLength(this, 'newHistoricContractDataTreeRootSiblingPath', CONTRACT_TREE_ROOTS_TREE_HEIGHT);
    assertLength(this, 'newL1ToL2MessageTreeRootSiblingPath', L1_TO_L2_MESSAGES_SIBLING_PATH_LENGTH);
    assertLength(this, 'newHistoricL1ToL2MessageTreeRootSiblingPath', L1_TO_L2_MESSAGES_ROOTS_TREE_HEIGHT);
    assertLength(this, 'newL1ToL2Messages', NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);
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
      fields.newHistoricPrivateDataTreeRootSiblingPath,
      fields.newHistoricContractDataTreeRootSiblingPath,
      fields.newL1ToL2Messages,
      fields.newL1ToL2MessageTreeRootSiblingPath,
      fields.newHistoricL1ToL2MessageTreeRootSiblingPath,
      fields.startL1ToL2MessageTreeSnapshot,
      fields.startHistoricTreeL1ToL2MessageTreeRootsSnapshot,
    ] as const;
  }
}

export class RootRollupPublicInputs {
  constructor(
    // NOTE: in practice, we'll hash all of this up into a single public input, for cheap on-chain verification.
    public endAggregationObject: AggregationObject,

    // constants: ConstantRollupData // TODO maybe don't include this

    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    public endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,

    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,

    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,

    public startPublicDataTreeRoot: Fr,
    public endPublicDataTreeRoot: Fr,

    public startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    public startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    public startL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,
    public endL1ToL2MessageTreeSnapshot: AppendOnlyTreeSnapshot,

    public startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    public calldataHash: [Fr, Fr],
    public l1ToL2MessagesHash: [Fr, Fr],
  ) {}

  static getFields(fields: FieldsOf<RootRollupPublicInputs>) {
    return [
      fields.endAggregationObject,
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

  static fromBuffer(buffer: Buffer | BufferReader): RootRollupPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new RootRollupPublicInputs(
      reader.readObject(AggregationObject),
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
      [reader.readFr(), reader.readFr()],
      [reader.readFr(), reader.readFr()],
    );
  }
}
