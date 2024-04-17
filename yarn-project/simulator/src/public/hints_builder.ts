import { MerkleTreeId } from '@aztec/circuit-types';
import {
  type Fr,
  type MAX_NEW_NULLIFIERS_PER_TX,
  type MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  type MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  type MAX_PUBLIC_DATA_HINTS,
  type MAX_PUBLIC_DATA_READS_PER_TX,
  type MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
  type PublicDataHint,
  type PublicDataRead,
  type PublicDataTreeLeafPreimage,
  type PublicDataUpdateRequest,
  type ReadRequestContext,
  type SideEffectLinkedToNoteHash,
  buildNullifierNonExistentReadRequestHints,
  buildNullifierReadRequestHints,
  buildPublicDataHints,
  buildPublicDataReadRequestHints,
} from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';
import { type IndexedTreeId, type MerkleTreeOperations } from '@aztec/world-state';

export class HintsBuilder {
  constructor(private db: MerkleTreeOperations) {}

  getNullifierReadRequestHints(
    nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    pendingNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {
    return buildNullifierReadRequestHints(this, nullifierReadRequests, pendingNullifiers);
  }

  getNullifierNonExistentReadRequestHints(
    nullifierNonExistentReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    pendingNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {
    return buildNullifierNonExistentReadRequestHints(this, nullifierNonExistentReadRequests, pendingNullifiers);
  }

  getPublicDataHints(
    publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
    publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
  ) {
    return buildPublicDataHints(this, publicDataReads, publicDataUpdateRequests);
  }

  getPublicDataReadRequestHints(
    publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
    publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    publicDataHints: Tuple<PublicDataHint, typeof MAX_PUBLIC_DATA_HINTS>,
  ) {
    return buildPublicDataReadRequestHints(publicDataReads, publicDataUpdateRequests, publicDataHints);
  }

  async getNullifierMembershipWitness(nullifier: Fr) {
    const index = await this.db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    if (index === undefined) {
      throw new Error(`Cannot find the leaf for nullifier ${nullifier.toBigInt()}.`);
    }

    return this.getMembershipWitnessWithPreimage<typeof NULLIFIER_TREE_HEIGHT>(
      MerkleTreeId.NULLIFIER_TREE,
      NULLIFIER_TREE_HEIGHT,
      index,
    );
  }

  async getLowNullifierMembershipWitness(nullifier: Fr) {
    const res = await this.db.getPreviousValueIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBigInt());
    if (!res) {
      throw new Error(`Cannot find the low leaf for nullifier ${nullifier.toBigInt()}.`);
    }

    const { index, alreadyPresent } = res;
    if (alreadyPresent) {
      throw new Error(`Nullifier ${nullifier.toBigInt()} already exists in the tree.`);
    }

    return this.getMembershipWitnessWithPreimage<typeof NULLIFIER_TREE_HEIGHT>(
      MerkleTreeId.NULLIFIER_TREE,
      NULLIFIER_TREE_HEIGHT,
      index,
    );
  }

  async getMatchOrLowPublicDataMembershipWitness(leafSlot: bigint) {
    const res = await this.db.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot);
    if (!res) {
      throw new Error(`Cannot find the previous value index for public data ${leafSlot}.`);
    }

    const { membershipWitness, leafPreimage } = await this.getMembershipWitnessWithPreimage<
      typeof PUBLIC_DATA_TREE_HEIGHT
    >(MerkleTreeId.PUBLIC_DATA_TREE, PUBLIC_DATA_TREE_HEIGHT, res.index);

    // Should find a way to stop casting IndexedTreeLeafPreimage as PublicDataTreeLeafPreimage everywhere.
    return { membershipWitness, leafPreimage: leafPreimage as PublicDataTreeLeafPreimage };
  }

  private async getMembershipWitnessWithPreimage<TREE_HEIGHT extends number>(
    treeId: IndexedTreeId,
    treeHeight: TREE_HEIGHT,
    index: bigint,
  ) {
    const siblingPath = await this.db.getSiblingPath(treeId, index);
    const membershipWitness = new MembershipWitness<TREE_HEIGHT>(treeHeight, index, siblingPath.toTuple());

    const leafPreimage = await this.db.getLeafPreimage(treeId, index);
    if (!leafPreimage) {
      throw new Error(`Cannot find the leaf preimage for tree ${treeId} at index ${index}.`);
    }

    return { membershipWitness, leafPreimage };
  }
}
