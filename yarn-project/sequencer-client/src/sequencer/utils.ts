import { CombinedHistoricTreeRoots, Fr, PrivateHistoricTreeRoots } from '@aztec/circuits.js';
import { MerkleTreeId, MerkleTreeOperations } from '@aztec/world-state';

export async function getCombinedHistoricTreeRoots(db: MerkleTreeOperations) {
  return new CombinedHistoricTreeRoots(
    new PrivateHistoricTreeRoots(
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.PRIVATE_DATA_TREE)).root),
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.NULLIFIER_TREE)).root),
      Fr.fromBuffer((await db.getTreeInfo(MerkleTreeId.CONTRACT_TREE)).root),
      Fr.ZERO,
    ),
  );
}
