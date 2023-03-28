import { Fr } from '@aztec/foundation';
import { assertLength, FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { AppendOnlyTreeSnapshot } from './base_rollup.js';
import { CONTRACT_TREE_HEIGHT, NULLIFIER_TREE_HEIGHT, PRIVATE_DATA_TREE_HEIGHT } from './constants.js';
import { PreviousRollupData } from './merge_rollup.js';
import { AggregationObject } from './shared.js';

export class RootRollupInputs {
  constructor(
    public previousRollupData: [PreviousRollupData, PreviousRollupData],

    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    // Note: the start_nullifier_tree_snapshot is contained within previous_rollup_data[0].public_inputs.start_nullifier_tree_snapshot.
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,

    // For inserting the new subtrees into their respective trees:
    // Note: the insertion leaf index can be derived from the above snapshots' `next_available_leaf_index` values.
    public newCommitmentsSubtreeSiblingPath: Fr[],
    public newNullifiersSubtreeSiblingPath: Fr[],
    public newContractsSubtreeSiblingPath: Fr[],

    public proverId: Fr,
  ) {
    assertLength(this, 'newCommitmentsSubtreeSiblingPath', PRIVATE_DATA_TREE_HEIGHT);
    assertLength(this, 'newNullifiersSubtreeSiblingPath', NULLIFIER_TREE_HEIGHT);
    assertLength(this, 'newContractsSubtreeSiblingPath', CONTRACT_TREE_HEIGHT);
  }

  toBuffer() {
    return serializeToBuffer(
      this.previousRollupData,
      this.startPrivateDataTreeSnapshot,
      this.startContractTreeSnapshot,
      this.newCommitmentsSubtreeSiblingPath,
      this.newNullifiersSubtreeSiblingPath,
      this.newContractsSubtreeSiblingPath,
      this.proverId,
    );
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

    // Hashes (probably sha256) to make public inputs constant-sized
    // (to then be unpacked on-chain)
    // UPDATE: we should instead just hash all of the below into a single value. See big diagram of sha256 hashing bottom-right of here.
    // TODO: I've put `fr`, but these hash values' types might need to be two fields if we want all 256-bits, for security purposes.
    public newCommitmentsHash: Fr,
    public newNullifiersHash: Fr,
    public newL1MsgsHash: Fr,
    public newContractDataHash: Fr,
    public proverContributionsHash: Fr, // TODO: spec how funds are distributed to provers.
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
      fields.newCommitmentsHash,
      fields.newNullifiersHash,
      fields.newL1MsgsHash,
      fields.newContractDataHash,
      fields.proverContributionsHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...RootRollupPublicInputs.getFields(this));
  }

  static from(fields: FieldsOf<RootRollupPublicInputs>): RootRollupPublicInputs {
    return new RootRollupPublicInputs(...RootRollupPublicInputs.getFields(fields));
  }
}
