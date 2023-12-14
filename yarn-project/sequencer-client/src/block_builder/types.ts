import { AppendOnlyTreeSnapshot, BaseOrMergeRollupPublicInputs, RootRollupPublicInputs } from '@aztec/circuits.js';

/**
 * Type representing the names of the trees for the base rollup.
 */
type BaseTreeNames = 'NoteHashTree' | 'ContractTree' | 'NullifierTree' | 'PublicDataTree';
/**
 * Type representing the names of the trees.
 */
export type TreeNames = BaseTreeNames | 'L1ToL2MessagesTree' | 'Archive';

/**
 * Type to assert that only the correct trees are checked when validating rollup tree outputs.
 */
export type AllowedTreeNames<T extends BaseOrMergeRollupPublicInputs | RootRollupPublicInputs> =
  T extends RootRollupPublicInputs ? TreeNames : BaseTreeNames;

/**
 * Type to assert the correct object field is indexed when validating rollup tree outputs.
 */
export type OutputWithTreeSnapshot<T extends BaseOrMergeRollupPublicInputs | RootRollupPublicInputs> = {
  [K in `end${AllowedTreeNames<T>}Snapshot`]: AppendOnlyTreeSnapshot;
};
