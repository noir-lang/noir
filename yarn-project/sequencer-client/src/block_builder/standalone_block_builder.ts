import { AppendOnlyTreeSnapshot, CircuitsWasm, NewContractData, makeEmptyProof } from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import { AztecAddress, Fr, createDebugLogger } from '@aztec/foundation';
import { ContractData, L2Block, PublicDataWrite } from '@aztec/types';
import { MerkleTreeId, MerkleTreeOperations } from '@aztec/world-state';
import { Proof } from '../prover/index.js';
import { ProcessedTx } from '../sequencer/processed_tx.js';
import { BlockBuilder } from './index.js';

const mapContractData = (n: NewContractData) => {
  const contractData = new ContractData(AztecAddress.fromBuffer(n.contractAddress.toBuffer()), n.portalContractAddress);
  return contractData;
};

/**
 * Builds an L2 block out of a tx by appending the commitments, nullifiers, and contracts to the trees.
 * @deprecated Use CircuitBlockBuilder instead.
 */
export class StandaloneBlockBuilder implements BlockBuilder {
  private dataTreeLeaves: Buffer[] = [];
  private nullifierTreeLeaves: Buffer[] = [];
  private contractTreeLeaves: Buffer[] = [];

  constructor(private db: MerkleTreeOperations, private log = createDebugLogger('aztec:block_builder')) {}

  /**
   * Creates a new L2Block with the given number, containing the set of processed txs, without calling any circuit.
   * @param blockNumber - Number of the block to assemble.
   * @param txs - Processed txs to include.
   * @param newL1ToL2Messages - L1 to L2 messages to be part of the block.
   * @returns The new L2 block along with an empty proof.
   */
  async buildL2Block(blockNumber: number, txs: ProcessedTx[], newL1ToL2Messages: Fr[]): Promise<[L2Block, Proof]> {
    const startPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    const startNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const startContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const startPublicDataTreeRoot = Fr.fromBuffer((await this.db.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE)).root);
    const startTreeOfHistoricPrivateDataTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
    );
    const startTreeOfHistoricContractTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
    );
    const startL1ToL2MessageTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGES_TREE);
    const startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE,
    );

    for (const tx of txs) {
      await this.updateTrees(tx);
    }
    await this.updateL1ToL2MessagesTree(newL1ToL2Messages);

    await this.updateRootTrees();

    const endPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    const endNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const endContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const endPublicDataTreeRoot = Fr.fromBuffer((await this.db.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE)).root);
    const endTreeOfHistoricPrivateDataTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
    );
    const endTreeOfHistoricContractTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
    );
    const endL1ToL2MessageTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGES_TREE);
    const endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE,
    );

    const l2Block = L2Block.fromFields({
      number: blockNumber,
      startPrivateDataTreeSnapshot,
      endPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      endNullifierTreeSnapshot,
      startContractTreeSnapshot,
      endContractTreeSnapshot,
      startPublicDataTreeRoot,
      endPublicDataTreeRoot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
      startL1ToL2MessageTreeSnapshot,
      endL1ToL2MessageTreeSnapshot,
      startTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      endTreeOfHistoricL1ToL2MessageTreeRootsSnapshot,
      newCommitments: this.dataTreeLeaves.map(b => Fr.fromBuffer(b)),
      newNullifiers: this.nullifierTreeLeaves.map(b => Fr.fromBuffer(b)),
      newContracts: this.contractTreeLeaves.map(b => Fr.fromBuffer(b)),
      newContractData: txs.flatMap(tx => tx.data.end.newContracts.map(mapContractData)),
      newPublicDataWrites: txs.flatMap(tx =>
        tx.data.end.stateTransitions.map(t => new PublicDataWrite(t.leafIndex, t.newValue)),
      ),
      newL1ToL2Messages,
    });
    return [l2Block, makeEmptyProof()];
  }

  private async getTreeSnapshot(id: MerkleTreeId): Promise<AppendOnlyTreeSnapshot> {
    const treeInfo = await this.db.getTreeInfo(id);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  }

  private async updateTrees(tx: ProcessedTx) {
    const wasm = await CircuitsWasm.get();
    const dataTreeLeaves = tx.data.end.newCommitments.map((x: Fr) => x.toBuffer());
    const nullifierTreeLeaves = tx.data.end.newNullifiers.map((x: Fr) => x.toBuffer());
    const contractTreeLeaves = tx.data.end.newContracts.map((x: NewContractData) =>
      computeContractLeaf(wasm, x).toBuffer(),
    );

    await this.db.appendLeaves(MerkleTreeId.PRIVATE_DATA_TREE, dataTreeLeaves);
    await this.db.appendLeaves(MerkleTreeId.NULLIFIER_TREE, nullifierTreeLeaves);
    await this.db.appendLeaves(MerkleTreeId.CONTRACT_TREE, contractTreeLeaves);
  }

  private async updateL1ToL2MessagesTree(l1ToL2Messages: Fr[]) {
    const leaves = l1ToL2Messages.map((x: Fr) => x.toBuffer());
    await this.db.appendLeaves(MerkleTreeId.L1_TO_L2_MESSAGES_TREE, leaves);
  }

  private async updateRootTrees() {
    const newDataTreeInfo = await this.getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    const newContractsTreeInfo = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const newL1ToL2MessagesTreeInfo = await this.getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGES_TREE);
    await this.db.appendLeaves(MerkleTreeId.CONTRACT_TREE_ROOTS_TREE, [newContractsTreeInfo.root.toBuffer()]);
    await this.db.appendLeaves(MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE, [newDataTreeInfo.root.toBuffer()]);
    await this.db.appendLeaves(MerkleTreeId.L1_TO_L2_MESSAGES_ROOTS_TREE, [newL1ToL2MessagesTreeInfo.root.toBuffer()]);
  }
}
