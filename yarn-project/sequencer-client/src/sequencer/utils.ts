import { CombinedHistoricTreeRoots, Fr, PrivateHistoricTreeRoots } from '@aztec/circuits.js';
import { MerkleTreeId } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';

/**
 * Fetches the private, nullifier, contract tree and l1 to l2 messages tree roots from a given db and assembles a CombinedHistoricTreeRoots object.
 */
export async function getCombinedHistoricTreeRoots(db: MerkleTreeOperations) {
  return new CombinedHistoricTreeRoots(
    new PrivateHistoricTreeRoots(
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.PRIVATE_DATA_TREE)).root),
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.NULLIFIER_TREE)).root),
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.CONTRACT_TREE)).root),
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.L1_TO_L2_MESSAGES_TREE)).root),
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.BLOCKS_TREE)).root),
      Fr.ZERO,
    ),
  );
}
