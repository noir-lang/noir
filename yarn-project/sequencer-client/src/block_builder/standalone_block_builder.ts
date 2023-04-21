import {
  AppendOnlyTreeSnapshot,
  CircuitsWasm,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  NewContractData,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import { AztecAddress, Fr, createDebugLogger } from '@aztec/foundation';
import { ContractData, L2Block } from '@aztec/types';
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

  async buildL2Block(blockNumber: number, txs: ProcessedTx[]): Promise<[L2Block, Proof]> {
    const startPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    const startNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const startContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const startTreeOfHistoricPrivateDataTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
    );
    const startTreeOfHistoricContractTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
    );

    for (const tx of txs) {
      await this.updateTrees(tx);
    }

    await this.updateRootTrees();

    const endPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    const endNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const endContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const endTreeOfHistoricPrivateDataTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE,
    );
    const endTreeOfHistoricContractTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
    );

    const l2Block = L2Block.fromFields({
      number: blockNumber,
      startPrivateDataTreeSnapshot,
      endPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      endNullifierTreeSnapshot,
      startContractTreeSnapshot,
      endContractTreeSnapshot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
      newCommitments: this.dataTreeLeaves.map(b => Fr.fromBuffer(b)),
      newNullifiers: this.nullifierTreeLeaves.map(b => Fr.fromBuffer(b)),
      newContracts: this.contractTreeLeaves.map(b => Fr.fromBuffer(b)),
      newContractData: txs.flatMap(tx => tx.data.end.newContracts.map(mapContractData)),
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

    for (let i = 0; i < KERNEL_NEW_COMMITMENTS_LENGTH; i++) {
      await this.db.appendLeaves(MerkleTreeId.PRIVATE_DATA_TREE, [dataTreeLeaves[i]]);
    }
    for (let i = 0; i < KERNEL_NEW_NULLIFIERS_LENGTH; i++) {
      await this.db.appendLeaves(MerkleTreeId.NULLIFIER_TREE, [nullifierTreeLeaves[i]]);
    }
    for (let i = 0; i < KERNEL_NEW_CONTRACTS_LENGTH; i++) {
      await this.db.appendLeaves(MerkleTreeId.CONTRACT_TREE, [contractTreeLeaves[i]]);
    }
  }

  private async updateRootTrees() {
    const newDataTreeInfo = await this.getTreeSnapshot(MerkleTreeId.PRIVATE_DATA_TREE);
    const newContractsTreeInfo = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    await this.db.appendLeaves(MerkleTreeId.CONTRACT_TREE_ROOTS_TREE, [newContractsTreeInfo.root.toBuffer()]);
    await this.db.appendLeaves(MerkleTreeId.PRIVATE_DATA_TREE_ROOTS_TREE, [newDataTreeInfo.root.toBuffer()]);
  }
}
