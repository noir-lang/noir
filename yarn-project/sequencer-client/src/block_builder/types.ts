import { AppendOnlyTreeSnapshot, BaseOrMergeRollupPublicInputs, RootRollupPublicInputs } from '@aztec/circuits.js';

/**
 * Type to assert that only the correct trees are checked when validating rollup tree outputs.
 */
export type AllowedTreeNames<T extends BaseOrMergeRollupPublicInputs | RootRollupPublicInputs> =
  T extends RootRollupPublicInputs
    ? 'PrivateData' | 'Contract' | 'Nullifier' | 'L1ToL2Messages' | 'HistoricBlocks'
    : 'PrivateData' | 'Contract' | 'Nullifier';

/**
 * Type to assert the correct object field is indexed when validating rollup tree outputs.
 */
export type OutputWithTreeSnapshot<T extends BaseOrMergeRollupPublicInputs | RootRollupPublicInputs> = {
  [K in `end${AllowedTreeNames<T>}TreeSnapshot`]: AppendOnlyTreeSnapshot;
};
