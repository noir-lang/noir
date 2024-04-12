import { type L2Block, type MerkleTreeId, type ProcessedTx, type ProvingResult } from '@aztec/circuit-types';
import {
  type AppendOnlyTreeSnapshot,
  type BaseOrMergeRollupPublicInputs,
  type BaseRollupInputs,
  type Fr,
  type GlobalVariables,
  type L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  type NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  type Proof,
  type RootParityInput,
  type RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';

export type MergeRollupInputData = {
  inputs: [BaseOrMergeRollupPublicInputs | undefined, BaseOrMergeRollupPublicInputs | undefined];
  proofs: [Proof | undefined, Proof | undefined];
};

enum PROVING_STATE_LIFECYCLE {
  PROVING_STATE_CREATED,
  PROVING_STATE_FULL,
  PROVING_STATE_RESOLVED,
  PROVING_STATE_REJECTED,
}

/**
 * The current state of the proving schedule. Contains the raw inputs (txs) and intermediate state to generate every constituent proof in the tree.
 * Carries an identifier so we can identify if the proving state is discarded and a new one started.
 * Captures resolve and reject callbacks to provide a promise base interface to the consumer of our proving.
 */
export class ProvingState {
  private provingStateLifecycle = PROVING_STATE_LIFECYCLE.PROVING_STATE_CREATED;
  private mergeRollupInputs: MergeRollupInputData[] = [];
  private rootParityInputs: Array<RootParityInput | undefined> = [];
  private finalRootParityInputs: RootParityInput | undefined;
  public rootRollupPublicInputs: RootRollupPublicInputs | undefined;
  public finalProof: Proof | undefined;
  public block: L2Block | undefined;
  private txs: ProcessedTx[] = [];
  public baseRollupInputs: BaseRollupInputs[] = [];
  public txTreeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>[] = [];
  constructor(
    public readonly totalNumTxs: number,
    private completionCallback: (result: ProvingResult) => void,
    private rejectionCallback: (reason: string) => void,
    public readonly globalVariables: GlobalVariables,
    public readonly newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
    numRootParityInputs: number,
    public readonly emptyTx: ProcessedTx,
    public readonly messageTreeSnapshot: AppendOnlyTreeSnapshot,
    public readonly messageTreeRootSiblingPath: Tuple<Fr, typeof L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH>,
  ) {
    this.rootParityInputs = Array.from({ length: numRootParityInputs }).map(_ => undefined);
  }

  public get baseMergeLevel() {
    return BigInt(Math.ceil(Math.log2(this.totalNumTxs)) - 1);
  }

  public get numMergeLevels() {
    return this.baseMergeLevel;
  }

  public addNewTx(tx: ProcessedTx) {
    this.txs.push(tx);
    if (this.txs.length === this.totalNumTxs) {
      this.provingStateLifecycle = PROVING_STATE_LIFECYCLE.PROVING_STATE_FULL;
    }
    return this.txs.length - 1;
  }

  public get transactionsReceived() {
    return this.txs.length;
  }

  public get finalRootParityInput() {
    return this.finalRootParityInputs;
  }

  public set finalRootParityInput(input: RootParityInput | undefined) {
    this.finalRootParityInputs = input;
  }

  public get rootParityInput() {
    return this.rootParityInputs;
  }

  public verifyState() {
    return (
      this.provingStateLifecycle === PROVING_STATE_LIFECYCLE.PROVING_STATE_CREATED ||
      this.provingStateLifecycle === PROVING_STATE_LIFECYCLE.PROVING_STATE_FULL
    );
  }

  public isAcceptingTransactions() {
    return this.provingStateLifecycle === PROVING_STATE_LIFECYCLE.PROVING_STATE_CREATED;
  }

  public get allTxs() {
    return this.txs;
  }

  public storeMergeInputs(
    mergeInputs: [BaseOrMergeRollupPublicInputs, Proof],
    indexWithinMerge: number,
    indexOfMerge: number,
  ) {
    if (!this.mergeRollupInputs[indexOfMerge]) {
      const mergeInputData: MergeRollupInputData = {
        inputs: [undefined, undefined],
        proofs: [undefined, undefined],
      };
      mergeInputData.inputs[indexWithinMerge] = mergeInputs[0];
      mergeInputData.proofs[indexWithinMerge] = mergeInputs[1];
      this.mergeRollupInputs[indexOfMerge] = mergeInputData;
      return false;
    }
    const mergeInputData = this.mergeRollupInputs[indexOfMerge];
    mergeInputData.inputs[indexWithinMerge] = mergeInputs[0];
    mergeInputData.proofs[indexWithinMerge] = mergeInputs[1];
    return true;
  }

  public getMergeInputs(indexOfMerge: number) {
    return this.mergeRollupInputs[indexOfMerge];
  }

  public isReadyForRootRollup() {
    return !(
      this.mergeRollupInputs[0] === undefined ||
      this.finalRootParityInput === undefined ||
      this.mergeRollupInputs[0].inputs.findIndex(p => !p) !== -1
    );
  }

  public setRootParityInputs(inputs: RootParityInput, index: number) {
    this.rootParityInputs[index] = inputs;
  }

  public areRootParityInputsReady() {
    return this.rootParityInputs.findIndex(p => !p) === -1;
  }

  public txHasPublicFunctions(index: number) {
    return index >= 0 && this.txs.length > index && this.txs[index].publicKernelRequests.length;
  }

  public getPublicFunction(txIndex: number, nextIndex: number) {
    if (txIndex < 0 || txIndex >= this.txs.length) {
      return undefined;
    }
    const tx = this.txs[txIndex];
    if (nextIndex < 0 || nextIndex >= tx.publicKernelRequests.length) {
      return undefined;
    }
    return tx.publicKernelRequests[nextIndex];
  }

  public cancel() {
    this.reject('Proving cancelled');
  }

  public reject(reason: string) {
    if (!this.verifyState()) {
      return;
    }
    this.provingStateLifecycle = PROVING_STATE_LIFECYCLE.PROVING_STATE_REJECTED;
    this.rejectionCallback(reason);
  }

  public resolve(result: ProvingResult) {
    if (!this.verifyState()) {
      return;
    }
    this.provingStateLifecycle = PROVING_STATE_LIFECYCLE.PROVING_STATE_RESOLVED;
    this.completionCallback(result);
  }
}
