import { BufferReader, Fr } from '@aztec/foundation';
import { assertLength, FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { CONTRACT_TREE_ROOTS_TREE_HEIGHT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT } from '../constants.js';
import { PreviousRollupData } from './previous_rollup_data.js';
import { AggregationObject } from '../aggregation_object.js';

export class RootRollupInputs {
  constructor(
    public previousRollupData: [PreviousRollupData, PreviousRollupData],

    public newHistoricPrivateDataTreeRootSiblingPath: Fr[],
    public newHistoricContractDataTreeRootSiblingPath: Fr[],
  ) {
    assertLength(this, 'newHistoricPrivateDataTreeRootSiblingPath', PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);
    assertLength(this, 'newHistoricContractDataTreeRootSiblingPath', CONTRACT_TREE_ROOTS_TREE_HEIGHT);
  }

  toBuffer() {
    return serializeToBuffer(
      this.previousRollupData,
      this.newHistoricPrivateDataTreeRootSiblingPath,
      this.newHistoricContractDataTreeRootSiblingPath,
    );
  }

  static from(fields: FieldsOf<RootRollupInputs>): RootRollupInputs {
    return new RootRollupInputs(...RootRollupInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<RootRollupInputs>) {
    return [
      fields.previousRollupData,
      fields.newHistoricPrivateDataTreeRootSiblingPath,
      fields.newHistoricContractDataTreeRootSiblingPath,
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

    public startTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public endTreeOfHistoricPrivateDataTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    public startTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,
    public endTreeOfHistoricContractTreeRootsSnapshot: AppendOnlyTreeSnapshot,

    public calldataHash: [Fr, Fr],
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
      fields.startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      fields.startTreeOfHistoricContractTreeRootsSnapshot,
      fields.endTreeOfHistoricContractTreeRootsSnapshot,
      fields.calldataHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...RootRollupPublicInputs.getFields(this));
  }

  static from(fields: FieldsOf<RootRollupPublicInputs>): RootRollupPublicInputs {
    return new RootRollupPublicInputs(...RootRollupPublicInputs.getFields(fields));
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
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      [reader.readFr(), reader.readFr()],
    );
  }
}
