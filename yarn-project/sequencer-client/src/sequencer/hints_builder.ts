import { MerkleTreeId } from '@aztec/circuit-types';
import {
  type Fr,
  MAX_NEW_NULLIFIERS_PER_TX,
  type MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
  type MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
  type PublicDataRead,
  PublicDataTreeLeafPreimage,
  type ReadRequestContext,
  type SideEffectLinkedToNoteHash,
  buildNullifierNonExistentReadRequestHints,
  buildNullifierReadRequestHints,
  mergeAccumulatedData,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';
import { type MerkleTreeOperations } from '@aztec/world-state';

export class HintsBuilder {
  constructor(private db: MerkleTreeOperations) {}

  getNullifierReadRequestHints(
    nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifiersNonRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    nullifiersRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {
    return buildNullifierReadRequestHints(
      this,
      nullifierReadRequests,
      mergeAccumulatedData(MAX_NEW_NULLIFIERS_PER_TX, nullifiersNonRevertible, nullifiersRevertible),
    );
  }

  getNullifierNonExistentReadRequestHints(
    nullifierNonExistentReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    nullifiersNonRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    nullifiersRevertible: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {
    const pendingNullifiers = mergeAccumulatedData(
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

  async getPublicDataReadsInfo(publicDataReads: PublicDataRead[]) {
    const newPublicDataReadsWitnesses: Tuple<
      MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_PUBLIC_DATA_READS_PER_TX
    > = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () => MembershipWitness.empty(PUBLIC_DATA_TREE_HEIGHT, 0n));

    const newPublicDataReadsPreimages: Tuple<PublicDataTreeLeafPreimage, typeof MAX_PUBLIC_DATA_READS_PER_TX> =
      makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () => PublicDataTreeLeafPreimage.empty());

    for (const i in publicDataReads) {
      const leafSlot = publicDataReads[i].leafSlot.value;
      const lowLeafResult = await this.db.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot);
      if (!lowLeafResult) {
        throw new Error(`Public data tree should have one initial leaf`);
      }
      const preimage = await this.db.getLeafPreimage(MerkleTreeId.PUBLIC_DATA_TREE, lowLeafResult.index);
      const path = await this.db.getSiblingPath(MerkleTreeId.PUBLIC_DATA_TREE, lowLeafResult.index);
      newPublicDataReadsWitnesses[i] = new MembershipWitness(
        PUBLIC_DATA_TREE_HEIGHT,
        BigInt(lowLeafResult.index),
        path.toTuple<typeof PUBLIC_DATA_TREE_HEIGHT>(),
      );
      newPublicDataReadsPreimages[i] = preimage! as PublicDataTreeLeafPreimage;
    }
    return {
      newPublicDataReadsWitnesses,
      newPublicDataReadsPreimages,
    };
  }
}
