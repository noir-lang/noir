import { type Fr } from '@aztec/foundation/fields';
import { type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import {
  type MAX_NEW_NULLIFIERS_PER_TX,
  type NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
  type NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
  type NULLIFIER_TREE_HEIGHT,
  type PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
} from '../../constants.gen.js';
import { type MembershipWitness } from '../membership_witness.js';
import { type NullifierLeafPreimage } from './nullifier_leaf/index.js';

/**
 * Hints used while proving state diff validity.
 */
export class StateDiffHints {
  constructor(
    /**
     * The nullifiers which need to be updated to perform the batch insertion of the new nullifiers.
     * See `StandardIndexedTree.batchInsert` function for more details.
     */
    public nullifierPredecessorPreimages: Tuple<NullifierLeafPreimage, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Membership witnesses for the nullifiers which need to be updated to perform the batch insertion of the new
     * nullifiers.
     */
    public nullifierPredecessorMembershipWitnesses: Tuple<
      MembershipWitness<typeof NULLIFIER_TREE_HEIGHT>,
      typeof MAX_NEW_NULLIFIERS_PER_TX
    >,
    /**
     * The nullifiers to be inserted in the tree, sorted high to low.
     */
    public sortedNullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The indexes of the sorted nullifiers to the original ones.
     */
    public sortedNullifierIndexes: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Sibling path "pointing to" where the new note hash subtree should be inserted into the note hash tree.
     */
    public noteHashSubtreeSiblingPath: Tuple<Fr, typeof NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Sibling path "pointing to" where the new nullifiers subtree should be inserted into the nullifier tree.
     */
    public nullifierSubtreeSiblingPath: Tuple<Fr, typeof NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH>,
    /**
     * Sibling path "pointing to" where the new public data subtree should be inserted into the public data tree.
     */
    public publicDataSiblingPath: Tuple<Fr, typeof PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH>,
  ) {}

  static from(fields: FieldsOf<StateDiffHints>): StateDiffHints {
    return new StateDiffHints(...StateDiffHints.getFields(fields));
  }

  static getFields(fields: FieldsOf<StateDiffHints>) {
    return [
      fields.nullifierPredecessorPreimages,
      fields.nullifierPredecessorMembershipWitnesses,
      fields.sortedNullifiers,
      fields.sortedNullifierIndexes,
      fields.noteHashSubtreeSiblingPath,
      fields.nullifierSubtreeSiblingPath,
      fields.publicDataSiblingPath,
    ] as const;
  }

  toBuffer(): Buffer {
    return serializeToBuffer(...StateDiffHints.getFields(this));
  }
}
