import { ContractData, L2Block } from '@aztec/types';
import {
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  AppendOnlyTreeSnapshot,
  NewContractData,
} from '@aztec/circuits.js';
import { MerkleTreeId, MerkleTreeOperations } from '@aztec/world-state';
import { Tx } from '@aztec/types';
import { AztecAddress, Fr, createDebugLogger } from '@aztec/foundation';

const mapContractData = (n: NewContractData) => {
  const contractData = new ContractData(AztecAddress.fromBuffer(n.contractAddress.toBuffer()), n.portalContractAddress);
  return contractData;
};

export class StandaloneBlockBuilder {
  private dataTreeLeaves: Buffer[] = [];
  private nullifierTreeLeaves: Buffer[] = [];
  private contractTreeLeaves: Buffer[] = [];

  constructor(
    private db: MerkleTreeOperations,
    private nextBlockNum: number,
    private tx: Tx,
    private log = createDebugLogger('aztec:block_builder'),
  ) {
    this.dataTreeLeaves = tx.data.end.newCommitments.map((x: Fr) => x.toBuffer());
    this.nullifierTreeLeaves = tx.data.end.newNullifiers.map((x: Fr) => x.toBuffer());
    this.contractTreeLeaves = tx.data.end.newContracts.map((x: NewContractData) => x.functionTreeRoot.toBuffer());
  }

  public async buildL2Block() {
    const startPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.DATA_TREE);
    const startNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const startContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const startTreeOfHistoricPrivateDataTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.DATA_TREE_ROOTS_TREE,
    );
    const startTreeOfHistoricContractTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
    );

    await this.updateTrees();

    const endPrivateDataTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.DATA_TREE);
    const endNullifierTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE);
    const endContractTreeSnapshot = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    const endTreeOfHistoricPrivateDataTreeRootsSnapshot = await this.getTreeSnapshot(MerkleTreeId.DATA_TREE_ROOTS_TREE);
    const endTreeOfHistoricContractTreeRootsSnapshot = await this.getTreeSnapshot(
      MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
    );
    this.log(`contract address ${this.tx.data.end.newContracts[0].contractAddress.toString()}`);

    const l2Block = L2Block.fromFields({
      number: this.nextBlockNum,
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
      newContractData: this.tx.data.end.newContracts.map(mapContractData),
    });
    return l2Block;
  }

  private async getTreeSnapshot(id: MerkleTreeId): Promise<AppendOnlyTreeSnapshot> {
    const treeInfo = await this.db.getTreeInfo(id);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  }

  private async updateTrees() {
    for (let i = 0; i < KERNEL_NEW_COMMITMENTS_LENGTH; i++) {
      await this.db.appendLeaves(MerkleTreeId.DATA_TREE, [this.dataTreeLeaves[i]]);
    }
    for (let i = 0; i < KERNEL_NEW_NULLIFIERS_LENGTH; i++) {
      await this.db.appendLeaves(MerkleTreeId.NULLIFIER_TREE, [this.nullifierTreeLeaves[i]]);
    }
    for (let i = 0; i < KERNEL_NEW_CONTRACTS_LENGTH; i++) {
      await this.db.appendLeaves(MerkleTreeId.CONTRACT_TREE, [this.contractTreeLeaves[i]]);
    }
    const newDataTreeInfo = await this.getTreeSnapshot(MerkleTreeId.DATA_TREE);
    const newContractsTreeInfo = await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE);
    await this.db.appendLeaves(MerkleTreeId.CONTRACT_TREE_ROOTS_TREE, [newContractsTreeInfo.root.toBuffer()]);
    await this.db.appendLeaves(MerkleTreeId.DATA_TREE_ROOTS_TREE, [newDataTreeInfo.root.toBuffer()]);
  }
}
