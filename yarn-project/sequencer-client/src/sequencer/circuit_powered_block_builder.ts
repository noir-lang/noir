import { ContractData, L2Block } from '@aztec/archiver';
import {
  AppendOnlyTreeSnapshot,
  BaseRollupInputs,
  BaseRollupPublicInputs,
  ConstantBaseRollupData,
  CONTRACT_TREE_ROOTS_TREE_HEIGHT,
  Fr,
  MembershipWitness,
  NullifierLeafPreimage,
  NULLIFIER_TREE_HEIGHT,
  PreviousKernelData,
  PreviousRollupData,
  PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
  RootRollupInputs,
  RootRollupPublicInputs,
  VK_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { Tx } from '@aztec/tx';
import { MerkleTreeId, MerkleTreeOperations } from '@aztec/world-state';
import { makeEmptyTx } from '../deps/tx.js';
import { Proof, Prover } from '../prover/index.js';
import { Simulator } from '../simulator/index.js';
import { VerificationKeys } from './vks.js';

// REFACTOR: Move this somewhere generic, and do something less horrible without going through hex strings.
const frToBigInt = (fr: Fr) => fr.value;
const bigintToFr = (num: bigint) => new Fr(num);
const bigintToNum = (num: bigint) => Number(num);

// Denotes fields that are not used now, but will be in the future
const FUTURE_FR = new Fr(0n);
const FUTURE_NUM = 0;

// Denotes fields that should be deleted
const DELETE_FR = new Fr(0n);
const DELETE_ANY: any = {};
const TODO_ANY: any = {};

export class CircuitPoweredBlockBuilder {
  constructor(
    private db: MerkleTreeOperations,
    private nextRollupId: number,
    private vks: VerificationKeys,
    private simulator: Simulator,
    private prover: Prover,
  ) {}

  public async buildL2Block(tx: Tx) {
    const [
      startPrivateDataTreeSnapshot,
      startNullifierTreeSnapshot,
      startContractTreeSnapshot,
      startTreeOfHistoricPrivateDataTreeRootsSnapshot,
      startTreeOfHistoricContractTreeRootsSnapshot,
    ] = await Promise.all(
      [
        MerkleTreeId.DATA_TREE,
        MerkleTreeId.NULLIFIER_TREE,
        MerkleTreeId.CONTRACT_TREE,
        MerkleTreeId.DATA_TREE_ROOTS_TREE,
        MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
      ].map(tree => this.getTreeSnapshot(tree)),
    );

    const [circuitsOutput] = await this.runCircuits(tx);

    const {
      endPrivateDataTreeSnapshot,
      endNullifierTreeSnapshot,
      endContractTreeSnapshot,
      endTreeOfHistoricPrivateDataTreeRootsSnapshot,
      endTreeOfHistoricContractTreeRootsSnapshot,
    } = circuitsOutput;

    const l2block = L2Block.fromFields({
      number: this.nextRollupId,
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
      newCommitments: tx.data.end.newCommitments,
      newNullifiers: tx.data.end.newNullifiers,
      newContracts: tx.data.end.newContracts.map(x => x.functionTreeRoot),
      newContractData: tx.data.end.newContracts.map(n => new ContractData(n.contractAddress, n.portalContractAddress)),
    });
    return l2block;
  }

  private async getTreeSnapshot(id: MerkleTreeId): Promise<AppendOnlyTreeSnapshot> {
    const treeInfo = await this.db.getTreeInfo(id);
    return new AppendOnlyTreeSnapshot(Fr.fromBuffer(treeInfo.root), Number(treeInfo.size));
  }

  private async runCircuits(tx: Tx): Promise<[RootRollupPublicInputs, Proof]> {
    const emptyTx = makeEmptyTx();

    const [, baseRollupOutputLeft, baseRollupProofLeft] = await this.baseRollupCircuit(tx, emptyTx);

    const [, baseRollupOutputRight, baseRollupProofRight] = await this.baseRollupCircuit(emptyTx, emptyTx);

    const rootInput = await this.getRootRollupInput(
      baseRollupOutputLeft,
      baseRollupProofLeft,
      baseRollupOutputRight,
      baseRollupProofRight,
    );
    const rootOutput = await this.simulator.rootRollupCircuit(rootInput);
    const rootProof = await this.prover.getRootRollupProof(rootInput, rootOutput);

    return [rootOutput, rootProof];
  }

  private async baseRollupCircuit(tx1: Tx, tx2: Tx) {
    const rollupInput = await this.getBaseRollupInput(tx1, tx2);
    const rollupOutput = await this.simulator.baseRollupCircuit(rollupInput);
    const rollupProof = await this.prover.getBaseRollupProof(rollupInput, rollupOutput);
    return [rollupInput, rollupOutput, rollupProof] as const;
  }

  private async getRootRollupInput(
    rollupOutputLeft: BaseRollupPublicInputs,
    rollupProofLeft: Proof,
    rollupOutputRight: BaseRollupPublicInputs,
    rollupProofRight: Proof,
  ) {
    const previousRollupData: RootRollupInputs['previousRollupData'] = [
      this.getPreviousRollupDataFromBaseRollup(rollupOutputLeft, rollupProofLeft),
      this.getPreviousRollupDataFromBaseRollup(rollupOutputRight, rollupProofRight),
    ];

    return new RootRollupInputs(
      previousRollupData,
      await this.getTreeSnapshot(MerkleTreeId.DATA_TREE),
      await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE),
      TODO_ANY,
      TODO_ANY,
      TODO_ANY,
    );
  }

  private getPreviousRollupDataFromBaseRollup(rollupOutput: BaseRollupPublicInputs, rollupProof: Proof) {
    return new PreviousRollupData(
      rollupOutput,
      rollupProof,
      this.vks.baseRollupCircuit,

      // MembershipWitness for a VK tree to be implemented in the future
      FUTURE_NUM,
      Array(VK_TREE_HEIGHT).fill(FUTURE_FR),
    );
  }

  private getKernelDataFor(tx: Tx) {
    return new PreviousKernelData(
      tx.data,
      tx.proof,

      // VK for the kernel circuit
      this.vks.kernelCircuit,

      // MembershipWitness for a VK tree to be implemented in the future
      FUTURE_NUM,
      Array(VK_TREE_HEIGHT).fill(FUTURE_FR),
    );
  }

  // Scan a tree searching for a specific value and return a membership witness proof for it
  private async getMembershipWitnessFor<N extends number>(
    value: Fr,
    treeId: MerkleTreeId,
    height: N,
  ): Promise<MembershipWitness<N>> {
    const index = await this.db.findLeafIndex(treeId, value.toBuffer());
    if (!index) throw new Error(`Leaf with value ${value} not found in tree ${treeId}`);
    const path = await this.db.getSiblingPath(treeId, index);
    // TODO: Check conversion from bigint to number
    return new MembershipWitness(
      height,
      Number(index),
      path.data.map(b => Fr.fromBuffer(b)),
    );
  }

  private async getConstantBaseRollupData(): Promise<ConstantBaseRollupData> {
    return ConstantBaseRollupData.from({
      baseRollupVkHash: DELETE_FR,
      mergeRollupVkHash: DELETE_FR,
      privateKernelVkTreeRoot: FUTURE_FR,
      publicKernelVkTreeRoot: FUTURE_FR,
      startTreeOfHistoricContractTreeRootsSnapshot: await this.getTreeSnapshot(MerkleTreeId.CONTRACT_TREE_ROOTS_TREE),
      startTreeOfHistoricPrivateDataTreeRootsSnapshot: await this.getTreeSnapshot(MerkleTreeId.DATA_TREE_ROOTS_TREE),
      treeOfHistoricL1ToL2MsgTreeRootsSnapshot: DELETE_ANY,
    });
  }

  private async getLowNullifierInfo(nullifier: Fr) {
    const tree = MerkleTreeId.NULLIFIER_TREE;
    const prevValueIndex = await this.db.getPreviousValueIndex(tree, frToBigInt(nullifier));
    const prevValueInfo = this.db.getLeafData(tree, prevValueIndex.index);
    if (!prevValueInfo) throw new Error(`Nullifier tree should have one initial leaf`);
    const prevValueSiblingPath = await this.db.getSiblingPath(tree, BigInt(prevValueIndex.index));
    return {
      index: prevValueIndex,
      leafPreimage: new NullifierLeafPreimage(
        bigintToFr(prevValueInfo.value),
        bigintToFr(prevValueInfo.nextValue),
        bigintToNum(prevValueInfo.nextIndex),
      ),
      witness: new MembershipWitness(
        NULLIFIER_TREE_HEIGHT,
        prevValueIndex.index,
        prevValueSiblingPath.data.map(b => Fr.fromBuffer(b)),
      ),
    };
  }

  private async getBaseRollupInput(tx1: Tx, tx2: Tx) {
    // Concatenate the new nullifiers of each tx being rolled up
    // and get the previous node for each of them, along with the sibling path
    const lowNullifierInfos = await Promise.all(
      [...tx1.data.end.newNullifiers, ...tx2.data.end.newNullifiers].map(fr => this.getLowNullifierInfo(fr)),
    );

    const getContractMembershipWitnessFor = (tx: Tx) =>
      this.getMembershipWitnessFor(
        tx.data.constants.oldTreeRoots.contractTreeRoot,
        MerkleTreeId.CONTRACT_TREE_ROOTS_TREE,
        CONTRACT_TREE_ROOTS_TREE_HEIGHT,
      );

    const getDataMembershipWitnessFor = (tx: Tx) =>
      this.getMembershipWitnessFor(
        tx.data.constants.oldTreeRoots.privateDataTreeRoot,
        MerkleTreeId.DATA_TREE_ROOTS_TREE,
        PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT,
      );

    return BaseRollupInputs.from({
      constants: await this.getConstantBaseRollupData(),
      startNullifierTreeSnapshot: await this.getTreeSnapshot(MerkleTreeId.NULLIFIER_TREE),
      lowNullifierLeafPreimages: lowNullifierInfos.map(i => i.leafPreimage),
      lowNullifierMembershipWitness: lowNullifierInfos.map(i => i.witness),
      kernelData: [this.getKernelDataFor(tx1), this.getKernelDataFor(tx2)],
      historicContractsTreeRootMembershipWitnesses: [
        await getContractMembershipWitnessFor(tx1),
        await getContractMembershipWitnessFor(tx2),
      ],
      historicPrivateDataTreeRootMembershipWitnesses: [
        await getDataMembershipWitnessFor(tx1),
        await getDataMembershipWitnessFor(tx2),
      ],
    } as BaseRollupInputs);
  }
}
