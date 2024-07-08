import { type L2Block, type MerkleTreeId, type ProvingResult } from '@aztec/circuit-types';
import {
  type AppendOnlyTreeSnapshot,
  type BaseOrMergeRollupPublicInputs,
  type Fr,
  type GlobalVariables,
  type L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  type Proof,
  type RECURSIVE_PROOF_LENGTH,
  type RecursiveProof,
  type RootParityInput,
  type RootRollupPublicInputs,
  type VerificationKeyAsFields,
} from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';

import { type TxProvingState } from './tx-proving-state.js';

export type MergeRollupInputData = {
  inputs: [BaseOrMergeRollupPublicInputs | undefined, BaseOrMergeRollupPublicInputs | undefined];
  proofs: [
    RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH> | undefined,
    RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH> | undefined,
  ];
  verificationKeys: [VerificationKeyAsFields | undefined, VerificationKeyAsFields | undefined];
};

export type TreeSnapshots = Map<MerkleTreeId, AppendOnlyTreeSnapshot>;

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
  private rootParityInputs: Array<RootParityInput<typeof RECURSIVE_PROOF_LENGTH> | undefined> = [];
  private finalRootParityInputs: RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH> | undefined;
  public rootRollupPublicInputs: RootRollupPublicInputs | undefined;
  public finalAggregationObject: Fr[] | undefined;
  public finalProof: Proof | undefined;
  public block: L2Block | undefined;
  private txs: TxProvingState[] = [];
  constructor(
    public readonly totalNumTxs: number,
    private completionCallback: (result: ProvingResult) => void,
    private rejectionCallback: (reason: string) => void,
    public readonly globalVariables: GlobalVariables,
    public readonly newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
    numRootParityInputs: number,
    public readonly messageTreeSnapshot: AppendOnlyTreeSnapshot,
    public readonly messageTreeRootSiblingPath: Tuple<Fr, typeof L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH>,
  ) {
    this.rootParityInputs = Array.from({ length: numRootParityInputs }).map(_ => undefined);
  }

  // Returns the number of levels of merge rollups
  public get numMergeLevels() {
    return BigInt(Math.ceil(Math.log2(this.totalNumTxs)) - 1);
  }

  // Calculates the index and level of the parent rollup circuit
  // Based on tree implementation in unbalanced_tree.ts -> batchInsert()
  public findMergeLevel(currentLevel: bigint, currentIndex: bigint) {
    const moveUpMergeLevel = (levelSize: number, index: bigint, nodeToShift: boolean) => {
      levelSize /= 2;
      if (levelSize & 1) {
        [levelSize, nodeToShift] = nodeToShift ? [levelSize + 1, false] : [levelSize - 1, true];
      }
      index >>= 1n;
      return { thisLevelSize: levelSize, thisIndex: index, shiftUp: nodeToShift };
    };
    let [thisLevelSize, shiftUp] = this.totalNumTxs & 1 ? [this.totalNumTxs - 1, true] : [this.totalNumTxs, false];
    const maxLevel = this.numMergeLevels + 1n;
    let placeholder = currentIndex;
    for (let i = 0; i < maxLevel - currentLevel; i++) {
      ({ thisLevelSize, thisIndex: placeholder, shiftUp } = moveUpMergeLevel(thisLevelSize, placeholder, shiftUp));
    }
    let thisIndex = currentIndex;
    let mergeLevel = currentLevel;
    while (thisIndex >= thisLevelSize && mergeLevel != 0n) {
      mergeLevel -= 1n;
      ({ thisLevelSize, thisIndex, shiftUp } = moveUpMergeLevel(thisLevelSize, thisIndex, shiftUp));
    }
    return [mergeLevel - 1n, thisIndex >> 1n, thisIndex & 1n];
  }

  // Adds a transaction to the proving state, returns it's index
  // Will update the proving life cycle if this is the last transaction
  public addNewTx(tx: TxProvingState) {
    this.txs.push(tx);
    if (this.txs.length === this.totalNumTxs) {
      this.provingStateLifecycle = PROVING_STATE_LIFECYCLE.PROVING_STATE_FULL;
    }
    return this.txs.length - 1;
  }

  // Returns the number of received transactions
  public get transactionsReceived() {
    return this.txs.length;
  }

  // Returns the final set of root parity inputs
  public get finalRootParityInput() {
    return this.finalRootParityInputs;
  }

  // Sets the final set of root parity inputs
  public set finalRootParityInput(input: RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH> | undefined) {
    this.finalRootParityInputs = input;
  }

  // Returns the set of root parity inputs
  public get rootParityInput() {
    return this.rootParityInputs;
  }

  // Returns true if this proving state is still valid, false otherwise
  public verifyState() {
    return (
      this.provingStateLifecycle === PROVING_STATE_LIFECYCLE.PROVING_STATE_CREATED ||
      this.provingStateLifecycle === PROVING_STATE_LIFECYCLE.PROVING_STATE_FULL
    );
  }

  // Returns true if we are still able to accept transactions, false otherwise
  public isAcceptingTransactions() {
    return this.provingStateLifecycle === PROVING_STATE_LIFECYCLE.PROVING_STATE_CREATED;
  }

  // Returns the complete set of transaction proving state objects
  public get allTxs() {
    return this.txs;
  }

  /**
   * Stores the inputs to a merge circuit and determines if the circuit is ready to be executed
   * @param mergeInputs - The inputs to store
   * @param indexWithinMerge - The index in the set of inputs to this merge circuit
   * @param indexOfMerge - The global index of this merge circuit
   * @returns True if the merge circuit is ready to be executed, false otherwise
   */
  public storeMergeInputs(
    mergeInputs: [
      BaseOrMergeRollupPublicInputs,
      RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>,
      VerificationKeyAsFields,
    ],
    indexWithinMerge: number,
    indexOfMerge: number,
  ) {
    if (!this.mergeRollupInputs[indexOfMerge]) {
      const mergeInputData: MergeRollupInputData = {
        inputs: [undefined, undefined],
        proofs: [undefined, undefined],
        verificationKeys: [undefined, undefined],
      };
      mergeInputData.inputs[indexWithinMerge] = mergeInputs[0];
      mergeInputData.proofs[indexWithinMerge] = mergeInputs[1];
      mergeInputData.verificationKeys[indexWithinMerge] = mergeInputs[2];
      this.mergeRollupInputs[indexOfMerge] = mergeInputData;
      return false;
    }
    const mergeInputData = this.mergeRollupInputs[indexOfMerge];
    mergeInputData.inputs[indexWithinMerge] = mergeInputs[0];
    mergeInputData.proofs[indexWithinMerge] = mergeInputs[1];
    mergeInputData.verificationKeys[indexWithinMerge] = mergeInputs[2];
    return true;
  }

  // Returns a specific transaction proving state
  public getTxProvingState(txIndex: number) {
    return this.txs[txIndex];
  }

  // Returns a set of merge rollup inputs
  public getMergeInputs(indexOfMerge: number) {
    return this.mergeRollupInputs[indexOfMerge];
  }

  // Returns true if we have sufficient inputs to execute the root rollup
  public isReadyForRootRollup() {
    return !(
      this.mergeRollupInputs[0] === undefined ||
      this.finalRootParityInput === undefined ||
      this.mergeRollupInputs[0].inputs.findIndex(p => !p) !== -1
    );
  }

  // Stores a set of root parity inputs at the given index
  public setRootParityInputs(inputs: RootParityInput<typeof RECURSIVE_PROOF_LENGTH>, index: number) {
    this.rootParityInputs[index] = inputs;
  }

  // Returns true if we have sufficient root parity inputs to execute the root parity circuit
  public areRootParityInputsReady() {
    return this.rootParityInputs.findIndex(p => !p) === -1;
  }

  // Attempts to reject the proving state promise with a reason of 'cancelled'
  public cancel() {
    this.reject('Proving cancelled');
  }

  // Attempts to reject the proving state promise with the given reason
  // Does nothing if not in a valid state
  public reject(reason: string) {
    if (!this.verifyState()) {
      return;
    }
    this.provingStateLifecycle = PROVING_STATE_LIFECYCLE.PROVING_STATE_REJECTED;
    this.rejectionCallback(reason);
  }

  // Attempts to resolve the proving state promise with the given result
  // Does nothing if not in a valid state
  public resolve(result: ProvingResult) {
    if (!this.verifyState()) {
      return;
    }
    this.provingStateLifecycle = PROVING_STATE_LIFECYCLE.PROVING_STATE_RESOLVED;
    this.completionCallback(result);
  }
}
