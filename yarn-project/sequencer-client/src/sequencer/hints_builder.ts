import { MerkleTreeId } from '@aztec/circuit-types';
import {
  Fr,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_REVERTIBLE_NULLIFIERS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  ReadRequestContext,
  SideEffectLinkedToNoteHash,
  buildNullifierNonExistentReadRequestHints,
  buildNullifierReadRequestHints,
  concatAccumulatedData,
} from '@aztec/circuits.js';
import { Tuple } from '@aztec/foundation/serialize';
import { MerkleTreeOperations } from '@aztec/world-state';

export class HintsBuilder {
  constructor(private db: MerkleTreeOperations) {}

  getNullifierReadRequestHints(
    nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifiersNonRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX>,
    nullifiersRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_REVERTIBLE_NULLIFIERS_PER_TX>,
  ) {
    return buildNullifierReadRequestHints(
      this,
      nullifierReadRequests,
      concatAccumulatedData(MAX_NEW_NULLIFIERS_PER_TX, nullifiersNonRevertible, nullifiersRevertible),
    );
  }

  getNullifierNonExistentReadRequestHints(
    nullifierNonExistentReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    nullifiersNonRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX>,
    nullifiersRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_REVERTIBLE_NULLIFIERS_PER_TX>,
  ) {
    const pendingNullifiers = concatAccumulatedData(
      MAX_NEW_NULLIFIERS_PER_TX,
      nullifiersNonRevertible,
      nullifiersRevertible,
    );
    return buildNullifierNonExistentReadRequestHints(this, nullifierNonExistentReadRequests, pendingNullifiers);
  }

  async getNullifierMembershipWitness(nullifier: Fr) {
    const index = await this.db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    if (index === undefined) {
      return;
    }

    return this.getNullifierMembershipWitnessWithPreimage(index);
  }

  async getLowNullifierMembershipWitness(nullifier: Fr) {
    const res = await this.db.getPreviousValueIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBigInt());
    if (res === undefined) {
      throw new Error(`Cannot find the low leaf for nullifier ${nullifier.toBigInt()}.`);
    }

    const { index, alreadyPresent } = res;
    if (alreadyPresent) {
      throw new Error(`Nullifier ${nullifier.toBigInt()} already exists in the tree.`);
    }

    return this.getNullifierMembershipWitnessWithPreimage(index);
  }

  private async getNullifierMembershipWitnessWithPreimage(index: bigint) {
    const siblingPath = await this.db.getSiblingPath<typeof NULLIFIER_TREE_HEIGHT>(MerkleTreeId.NULLIFIER_TREE, index);
    const membershipWitness = new MembershipWitness(
      NULLIFIER_TREE_HEIGHT,
      index,
      siblingPath.toTuple<typeof NULLIFIER_TREE_HEIGHT>(),
    );

    const leafPreimage = await this.db.getLeafPreimage(MerkleTreeId.NULLIFIER_TREE, index);
    if (!leafPreimage) {
      throw new Error(`Cannot find the leaf preimage at index ${index}.`);
    }

    return { membershipWitness, leafPreimage };
  }
}
