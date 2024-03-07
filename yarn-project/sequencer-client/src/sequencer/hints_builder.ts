import { MerkleTreeId } from '@aztec/circuit-types';
import {
  Fr,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_REVERTIBLE_NULLIFIERS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  NullifierLeafPreimage,
  ReadRequestContext,
  SideEffectLinkedToNoteHash,
  buildNullifierReadRequestResetHints,
  concatAccumulatedData,
  mergeAccumulatedData,
} from '@aztec/circuits.js';
import { Tuple } from '@aztec/foundation/serialize';
import { MerkleTreeOperations } from '@aztec/world-state';

export class HintsBuilder {
  constructor(private db: MerkleTreeOperations) {}

  getNullifierReadRequestResetHints(
    nullifierReadRequestsNonRevertible: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifierReadRequestsRevertible: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifiersNonRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX>,
    nullifiersRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_REVERTIBLE_NULLIFIERS_PER_TX>,
  ) {
    return buildNullifierReadRequestResetHints(
      this,
      mergeAccumulatedData(
        MAX_NULLIFIER_READ_REQUESTS_PER_TX,
        nullifierReadRequestsNonRevertible,
        nullifierReadRequestsRevertible,
      ),
      concatAccumulatedData(MAX_NEW_NULLIFIERS_PER_TX, nullifiersNonRevertible, nullifiersRevertible),
    );
  }

  async getNullifierMembershipWitness(nullifier: Fr) {
    const index = await this.db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    if (index === undefined) {
      return;
    }

    const siblingPath = await this.db.getSiblingPath(MerkleTreeId.NULLIFIER_TREE, index);
    const membershipWitness = new MembershipWitness(
      NULLIFIER_TREE_HEIGHT,
      index,
      siblingPath.toTuple<typeof NULLIFIER_TREE_HEIGHT>(),
    );
    const leafPreimage = (await this.db.getLeafPreimage(MerkleTreeId.NULLIFIER_TREE, index))! as NullifierLeafPreimage;

    return { membershipWitness, leafPreimage };
  }
}
