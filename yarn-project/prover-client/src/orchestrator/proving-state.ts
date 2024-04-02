import { type ProcessedTx, type ProvingResult } from '@aztec/circuit-types';
import {
  type BaseOrMergeRollupPublicInputs,
  type Fr,
  type GlobalVariables,
  type NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  type Proof,
  type RootParityInput,
} from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { type Tuple } from '@aztec/foundation/serialize';

/**
 * Enums and structs to communicate the type of work required in each request.
 */
export enum PROVING_JOB_TYPE {
  STATE_UPDATE,
  BASE_ROLLUP,
  MERGE_ROLLUP,
  ROOT_ROLLUP,
  BASE_PARITY,
  ROOT_PARITY,
}

export type ProvingJob = {
  type: PROVING_JOB_TYPE;
  operation: () => Promise<void>;
};

export type MergeRollupInputData = {
  inputs: [BaseOrMergeRollupPublicInputs | undefined, BaseOrMergeRollupPublicInputs | undefined];
  proofs: [Proof | undefined, Proof | undefined];
};

/**
 * The current state of the proving schedule. Contains the raw inputs (txs) and intermediate state to generate every constituent proof in the tree.
 * Carries an identifier so we can identify if the proving state is discarded and a new one started.
 * Captures resolve and reject callbacks to provide a promise base interface to the consumer of our proving.
 */
export class ProvingState {
  private stateIdentifier: string;
  private mergeRollupInputs: MergeRollupInputData[] = [];
  private rootParityInputs: Array<RootParityInput | undefined> = [];
  private finalRootParityInputs: RootParityInput | undefined;
  private finished = false;
  private txs: ProcessedTx[] = [];
  constructor(
    public readonly numTxs: number,
    private completionCallback: (result: ProvingResult) => void,
    private rejectionCallback: (reason: string) => void,
    public readonly globalVariables: GlobalVariables,
    public readonly newL1ToL2Messages: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>,
    numRootParityInputs: number,
    public readonly emptyTx: ProcessedTx,
  ) {
    this.stateIdentifier = randomBytes(32).toString('hex');
    this.rootParityInputs = Array.from({ length: numRootParityInputs }).map(_ => undefined);
  }

  public get baseMergeLevel() {
    return BigInt(Math.ceil(Math.log2(this.totalNumTxs)) - 1);
  }

  public get numMergeLevels() {
    return this.baseMergeLevel;
  }

  public get Id() {
    return this.stateIdentifier;
  }

  public get numPaddingTxs() {
    return this.totalNumTxs - this.numTxs;
  }

  public get totalNumTxs() {
    const realTxs = Math.max(2, this.numTxs);
    const pow2Txs = Math.ceil(Math.log2(realTxs));
    return 2 ** pow2Txs;
  }

  public addNewTx(tx: ProcessedTx) {
    this.txs.push(tx);
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

  public verifyState(stateId: string) {
    return stateId === this.stateIdentifier && !this.finished;
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
    if (this.mergeRollupInputs[0] === undefined) {
      return false;
    }
    if (this.mergeRollupInputs[0].inputs.findIndex(p => !p) !== -1) {
      return false;
    }
    if (this.finalRootParityInput === undefined) {
      return false;
    }
    return true;
  }

  public setRootParityInputs(inputs: RootParityInput, index: number) {
    this.rootParityInputs[index] = inputs;
  }

  public areRootParityInputsReady() {
    return this.rootParityInputs.findIndex(p => !p) === -1;
  }

  public reject(reason: string, stateIdentifier: string) {
    if (!this.verifyState(stateIdentifier)) {
      return;
    }
    if (this.finished) {
      return;
    }
    this.finished = true;
    this.rejectionCallback(reason);
  }

  public resolve(result: ProvingResult, stateIdentifier: string) {
    if (!this.verifyState(stateIdentifier)) {
      return;
    }
    if (this.finished) {
      return;
    }
    this.finished = true;
    this.completionCallback(result);
  }

  public isFinished() {
    return this.finished;
  }
}
