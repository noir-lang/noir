import {
  Body,
  L2Block,
  MerkleTreeId,
  type ProcessedTx,
  PublicKernelType,
  type TxEffect,
  toTxEffect,
} from '@aztec/circuit-types';
import {
  type BlockResult,
  PROVING_STATUS,
  type ProvingResult,
  type ProvingTicket,
} from '@aztec/circuit-types/interfaces';
import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  type BaseRollupInputs,
  Fr,
  type GlobalVariables,
  L1_TO_L2_MSG_SUBTREE_HEIGHT,
  L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  type Proof,
  RootParityInput,
  RootParityInputs,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { MemoryFifo } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';
import { sleep } from '@aztec/foundation/sleep';
import { elapsed } from '@aztec/foundation/timer';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { inspect } from 'util';

import { type CircuitProver } from '../prover/index.js';
import {
  buildBaseRollupInput,
  createMergeRollupInputs,
  getRootRollupInput,
  getSubtreeSiblingPath,
  getTreeSnapshot,
  validatePartialState,
  validateRootOutput,
  validateTx,
} from './block-building-helpers.js';
import { type MergeRollupInputData, ProvingState, type TreeSnapshots } from './proving-state.js';
import { TX_PROVING_CODE, TxProvingState } from './tx-proving-state.js';

const logger = createDebugLogger('aztec:prover:proving-orchestrator');

/**
 * Implements an event driven proving scheduler to build the recursive proof tree. The idea being:
 * 1. Transactions are provided to the scheduler post simulation.
 * 2. Tree insertions are performed as required to generate transaction specific proofs
 * 3. Those transaction specific proofs are generated in the necessary order accounting for dependencies
 * 4. Once a transaction is proven, it will be incorporated into a merge proof
 * 5. Merge proofs are produced at each level of the tree until the root proof is produced
 *
 * The proving implementation is determined by the provided prover. This could be for example a local prover or a remote prover pool.
 */

const SLEEP_TIME = 50;
const MAX_CONCURRENT_JOBS = 64;

enum PROMISE_RESULT {
  SLEEP,
  OPERATIONS,
}

const KernelTypesWithoutFunctions: Set<PublicKernelType> = new Set<PublicKernelType>([
  PublicKernelType.NON_PUBLIC,
  PublicKernelType.TAIL,
]);

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
  PUBLIC_KERNEL,
  PUBLIC_VM,
}

export type ProvingJob = {
  type: PROVING_JOB_TYPE;
  operation: () => Promise<void>;
};

/**
 * The orchestrator, managing the flow of recursive proving operations required to build the rollup proof tree.
 */
export class ProvingOrchestrator {
  private provingState: ProvingState | undefined = undefined;
  private jobQueue: MemoryFifo<ProvingJob> = new MemoryFifo<ProvingJob>();
  private jobProcessPromise?: Promise<void>;
  private stopped = false;
  constructor(
    private db: MerkleTreeOperations,
    private prover: CircuitProver,
    private maxConcurrentJobs = MAX_CONCURRENT_JOBS,
  ) {}

  // Constructs and starts a new orchestrator
  public static async new(db: MerkleTreeOperations, prover: CircuitProver) {
    const orchestrator = new ProvingOrchestrator(db, prover);
    await orchestrator.start();
    return Promise.resolve(orchestrator);
  }

  // Starts the proving job queue
  public start() {
    this.jobProcessPromise = this.processJobQueue();
    return Promise.resolve();
  }

  // Stops the proving job queue
  public async stop() {
    this.stopped = true;
    this.jobQueue.cancel();
    await this.jobProcessPromise;
  }

  /**
   * Starts off a new block
   * @param numTxs - The total number of transactions in the block. Must be a power of 2
   * @param globalVariables - The global variables for the block
   * @param l1ToL2Messages - The l1 to l2 messages for the block
   * @param emptyTx - The instance of an empty transaction to be used to pad this block
   * @returns A proving ticket, containing a promise notifying of proving completion
   */
  public async startNewBlock(
    numTxs: number,
    globalVariables: GlobalVariables,
    l1ToL2Messages: Fr[],
    emptyTx: ProcessedTx,
  ): Promise<ProvingTicket> {
    // Check that the length of the array of txs is a power of two
    // See https://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
    if (!Number.isInteger(numTxs) || numTxs < 2 || (numTxs & (numTxs - 1)) !== 0) {
      throw new Error(`Length of txs for the block should be a power of two and at least two (got ${numTxs})`);
    }
    // Cancel any currently proving block before starting a new one
    this.cancelBlock();
    logger.info(`Starting new block with ${numTxs} transactions`);
    // we start the block by enqueueing all of the base parity circuits
    let baseParityInputs: BaseParityInputs[] = [];
    let l1ToL2MessagesPadded: Tuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>;
    try {
      l1ToL2MessagesPadded = padArrayEnd(l1ToL2Messages, Fr.ZERO, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);
    } catch (err) {
      throw new Error('Too many L1 to L2 messages');
    }
    baseParityInputs = Array.from({ length: NUM_BASE_PARITY_PER_ROOT_PARITY }, (_, i) =>
      BaseParityInputs.fromSlice(l1ToL2MessagesPadded, i),
    );

    const messageTreeSnapshot = await getTreeSnapshot(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, this.db);

    const newL1ToL2MessageTreeRootSiblingPathArray = await getSubtreeSiblingPath(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      L1_TO_L2_MSG_SUBTREE_HEIGHT,
      this.db,
    );

    const newL1ToL2MessageTreeRootSiblingPath = makeTuple(
      L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
      i =>
        i < newL1ToL2MessageTreeRootSiblingPathArray.length ? newL1ToL2MessageTreeRootSiblingPathArray[i] : Fr.ZERO,
      0,
    );

    // Update the local trees to include the new l1 to l2 messages
    await this.db.appendLeaves(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, l1ToL2MessagesPadded);

    let provingState: ProvingState | undefined = undefined;

    const promise = new Promise<ProvingResult>((resolve, reject) => {
      provingState = new ProvingState(
        numTxs,
        resolve,
        reject,
        globalVariables,
        l1ToL2MessagesPadded,
        baseParityInputs.length,
        emptyTx,
        messageTreeSnapshot,
        newL1ToL2MessageTreeRootSiblingPath,
      );
    }).catch((reason: string) => ({ status: PROVING_STATUS.FAILURE, reason } as const));

    for (let i = 0; i < baseParityInputs.length; i++) {
      this.enqueueJob(provingState, PROVING_JOB_TYPE.BASE_PARITY, () =>
        this.runBaseParityCircuit(provingState, baseParityInputs[i], i),
      );
    }

    this.provingState = provingState;

    const ticket: ProvingTicket = {
      provingPromise: promise,
    };
    return ticket;
  }

  /**
   * The interface to add a simulated transaction to the scheduler
   * @param tx - The transaction to be proven
   */
  public async addNewTx(tx: ProcessedTx): Promise<void> {
    if (!this.provingState) {
      throw new Error(`Invalid proving state, call startNewBlock before adding transactions`);
    }

    if (!this.provingState.isAcceptingTransactions()) {
      throw new Error(`Rollup not accepting further transactions`);
    }

    validateTx(tx);

    logger.info(`Received transaction: ${tx.hash}`);

    await this.startTransaction(tx, this.provingState);
  }

  /**
   * Marks the block as full and pads it to the full power of 2 block size, no more transactions will be accepted.
   */
  public async setBlockCompleted() {
    if (!this.provingState) {
      throw new Error(`Invalid proving state, call startNewBlock before adding transactions or completing the block`);
    }

    // we need to pad the rollup with empty transactions
    logger.info(
      `Padding rollup with ${
        this.provingState.totalNumTxs - this.provingState.transactionsReceived
      } empty transactions`,
    );
    for (let i = this.provingState.transactionsReceived; i < this.provingState.totalNumTxs; i++) {
      await this.startTransaction(this.provingState.emptyTx, this.provingState);
    }
  }

  /**
   * Cancel any further proving of the block
   */
  public cancelBlock() {
    this.provingState?.cancel();
  }

  /**
   * Performs the final tree update for the block and returns the fully proven block.
   * @returns The fully proven block and proof.
   */
  public async finaliseBlock() {
    if (!this.provingState || !this.provingState.rootRollupPublicInputs || !this.provingState.finalProof) {
      throw new Error(`Invalid proving state, a block must be proven before it can be finalised`);
    }
    if (this.provingState.block) {
      throw new Error('Block already finalised');
    }

    const rootRollupOutputs = this.provingState.rootRollupPublicInputs;

    logger?.debug(`Updating and validating root trees`);
    await this.db.updateArchive(rootRollupOutputs.header);

    await validateRootOutput(rootRollupOutputs, this.db);

    // Collect all new nullifiers, commitments, and contracts from all txs in this block
    const nonEmptyTxEffects: TxEffect[] = this.provingState!.allTxs.map(txProvingState =>
      toTxEffect(txProvingState.processedTx),
    ).filter(txEffect => !txEffect.isEmpty());
    const blockBody = new Body(nonEmptyTxEffects);

    const l2Block = L2Block.fromFields({
      archive: rootRollupOutputs.archive,
      header: rootRollupOutputs.header,
      body: blockBody,
    });

    if (!l2Block.body.getTxsEffectsHash().equals(rootRollupOutputs.header.contentCommitment.txsEffectsHash)) {
      logger.debug(inspect(blockBody));
      throw new Error(
        `Txs effects hash mismatch, ${l2Block.body
          .getTxsEffectsHash()
          .toString('hex')} == ${rootRollupOutputs.header.contentCommitment.txsEffectsHash.toString('hex')} `,
      );
    }

    logger.info(`Successfully proven block ${l2Block.number}!`);

    this.provingState.block = l2Block;

    const blockResult: BlockResult = {
      proof: this.provingState.finalProof,
      block: l2Block,
    };

    return blockResult;
  }

  /**
   * Starts the proving process for the given transaction and adds it to our state
   * @param tx - The transaction whose proving we wish to commence
   * @param provingState - The proving state being worked on
   */
  private async startTransaction(tx: ProcessedTx, provingState: ProvingState) {
    const txInputs = await this.prepareBaseRollupInputs(provingState, tx);
    if (!txInputs) {
      // This should not be possible
      throw new Error(`Unable to add padding transaction, preparing base inputs failed`);
    }
    const [inputs, treeSnapshots] = txInputs;
    const txProvingState = new TxProvingState(tx, inputs, treeSnapshots);
    const txIndex = provingState.addNewTx(txProvingState);
    const numPublicKernels = txProvingState.getNumPublicKernels();
    if (!numPublicKernels) {
      // no public functions, go straight to the base rollup
      logger.debug(`Enqueueing base rollup for tx ${txIndex}`);
      this.enqueueJob(provingState, PROVING_JOB_TYPE.BASE_ROLLUP, () =>
        this.runBaseRollup(provingState, BigInt(txIndex), txProvingState),
      );
      return;
    }
    // Enqueue all of the VM proving requests
    // Rather than handle the Kernel Tail as a special case here, we will just handle it inside executeVM
    for (let i = 0; i < numPublicKernels; i++) {
      logger.debug(`Enqueueing public VM ${i} for tx ${txIndex}`);
      this.enqueueJob(provingState, PROVING_JOB_TYPE.PUBLIC_VM, () => this.executeVM(provingState, txIndex, i));
    }
  }

  /**
   * Enqueue a job to be scheduled
   * @param provingState - The proving state object being operated on
   * @param jobType - The type of job to be queued
   * @param job - The actual job, returns a promise notifying of the job's completion
   */
  private enqueueJob(provingState: ProvingState | undefined, jobType: PROVING_JOB_TYPE, job: () => Promise<void>) {
    if (!provingState?.verifyState()) {
      logger.debug(`Not enqueueing job, proving state invalid`);
      return;
    }
    // We use a 'safeJob'. We don't want promise rejections in the proving pool, we want to capture the error here
    // and reject the proving job whilst keeping the event loop free of rejections
    const safeJob = async () => {
      try {
        await job();
      } catch (err) {
        logger.error(`Error thrown when proving job type ${PROVING_JOB_TYPE[jobType]}: ${err}`);
        provingState!.reject(`${err}`);
      }
    };
    const provingJob: ProvingJob = {
      type: jobType,
      operation: safeJob,
    };
    this.jobQueue.put(provingJob);
  }

  // Updates the merkle trees for a transaction. The first enqueued job for a transaction
  private async prepareBaseRollupInputs(
    provingState: ProvingState | undefined,
    tx: ProcessedTx,
  ): Promise<[BaseRollupInputs, TreeSnapshots] | undefined> {
    if (!provingState?.verifyState()) {
      logger.debug('Not preparing base rollup inputs, state invalid');
      return;
    }
    const inputs = await buildBaseRollupInput(tx, provingState.globalVariables, this.db);
    const promises = [MerkleTreeId.NOTE_HASH_TREE, MerkleTreeId.NULLIFIER_TREE, MerkleTreeId.PUBLIC_DATA_TREE].map(
      async (id: MerkleTreeId) => {
        return { key: id, value: await getTreeSnapshot(id, this.db) };
      },
    );
    const treeSnapshots: TreeSnapshots = new Map((await Promise.all(promises)).map(obj => [obj.key, obj.value]));

    if (!provingState?.verifyState()) {
      logger.debug(`Discarding proving job, state no longer valid`);
      return;
    }
    return [inputs, treeSnapshots];
  }

  // Stores the intermediate inputs prepared for a merge proof
  private storeMergeInputs(
    provingState: ProvingState,
    currentLevel: bigint,
    currentIndex: bigint,
    mergeInputs: [BaseOrMergeRollupPublicInputs, Proof],
  ) {
    const mergeLevel = currentLevel - 1n;
    const indexWithinMergeLevel = currentIndex >> 1n;
    const mergeIndex = 2n ** mergeLevel - 1n + indexWithinMergeLevel;
    const subscript = Number(mergeIndex);
    const indexWithinMerge = Number(currentIndex & 1n);
    const ready = provingState.storeMergeInputs(mergeInputs, indexWithinMerge, subscript);
    return { ready, indexWithinMergeLevel, mergeLevel, mergeInputData: provingState.getMergeInputs(subscript) };
  }

  // Executes the base rollup circuit and stored the output as intermediate state for the parent merge/root circuit
  // Executes the next level of merge if all inputs are available
  private async runBaseRollup(provingState: ProvingState | undefined, index: bigint, tx: TxProvingState) {
    if (
      !tx.baseRollupInputs.kernelData.publicInputs.end.encryptedLogsHash
        .toBuffer()
        .equals(tx.processedTx.encryptedLogs.hash())
    ) {
      throw new Error(
        `Encrypted logs hash mismatch: ${
          tx.baseRollupInputs.kernelData.publicInputs.end.encryptedLogsHash
        } === ${Fr.fromBuffer(tx.processedTx.encryptedLogs.hash())}`,
      );
    }
    if (
      !tx.baseRollupInputs.kernelData.publicInputs.end.unencryptedLogsHash
        .toBuffer()
        .equals(tx.processedTx.unencryptedLogs.hash())
    ) {
      throw new Error(
        `Unencrypted logs hash mismatch: ${
          tx.baseRollupInputs.kernelData.publicInputs.end.unencryptedLogsHash
        } === ${Fr.fromBuffer(tx.processedTx.unencryptedLogs.hash())}`,
      );
    }
    if (!provingState?.verifyState()) {
      logger.debug('Not running base rollup, state invalid');
      return;
    }
    const [duration, baseRollupOutputs] = await elapsed(async () => {
      const [rollupOutput, proof] = await this.prover.getBaseRollupProof(tx.baseRollupInputs);
      validatePartialState(rollupOutput.end, tx.treeSnapshots);
      return { rollupOutput, proof };
    });
    logger.debug(`Simulated base rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'base-rollup',
      duration,
      inputSize: tx.baseRollupInputs.toBuffer().length,
      outputSize: baseRollupOutputs.rollupOutput.toBuffer().length,
    } satisfies CircuitSimulationStats);
    if (!provingState?.verifyState()) {
      logger.debug(`Discarding job as state no longer valid`);
      return;
    }
    const currentLevel = provingState.numMergeLevels + 1n;
    logger.info(`Completed base rollup at index ${index}, current level ${currentLevel}`);
    this.storeAndExecuteNextMergeLevel(provingState, currentLevel, index, [
      baseRollupOutputs.rollupOutput,
      baseRollupOutputs.proof,
    ]);
  }

  // Executes the merge rollup circuit and stored the output as intermediate state for the parent merge/root circuit
  // Enqueues the next level of merge if all inputs are available
  private async runMergeRollup(
    provingState: ProvingState | undefined,
    level: bigint,
    index: bigint,
    mergeInputData: MergeRollupInputData,
  ) {
    if (!provingState?.verifyState()) {
      logger.debug('Not running merge rollup, state invalid');
      return;
    }
    const circuitInputs = createMergeRollupInputs(
      [mergeInputData.inputs[0]!, mergeInputData.proofs[0]!],
      [mergeInputData.inputs[1]!, mergeInputData.proofs[1]!],
    );
    const [duration, circuitOutputs] = await elapsed(() => this.prover.getMergeRollupProof(circuitInputs));
    logger.debug(`Simulated merge rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'merge-rollup',
      duration,
      inputSize: circuitInputs.toBuffer().length,
      outputSize: circuitOutputs[0].toBuffer().length,
    } satisfies CircuitSimulationStats);
    if (!provingState?.verifyState()) {
      logger.debug(`Discarding job as state no longer valid`);
      return;
    }
    logger.info(`Completed merge rollup at level ${level}, index ${index}`);
    this.storeAndExecuteNextMergeLevel(provingState, level, index, circuitOutputs);
  }

  // Executes the root rollup circuit
  private async runRootRollup(provingState: ProvingState | undefined) {
    if (!provingState?.verifyState()) {
      logger.debug('Not running root rollup, state no longer valid');
      return;
    }
    const mergeInputData = provingState.getMergeInputs(0);
    const rootParityInput = provingState.finalRootParityInput!;

    const rootInput = await getRootRollupInput(
      mergeInputData.inputs[0]!,
      mergeInputData.proofs[0]!,
      mergeInputData.inputs[1]!,
      mergeInputData.proofs[1]!,
      rootParityInput,
      provingState.newL1ToL2Messages,
      provingState.messageTreeSnapshot,
      provingState.messageTreeRootSiblingPath,
      this.db,
    );

    // Simulate and get proof for the root circuit
    const [rootOutput, rootProof] = await this.prover.getRootRollupProof(rootInput);

    logger.info(`Completed root rollup`);

    provingState.rootRollupPublicInputs = rootOutput;
    provingState.finalProof = rootProof;

    const provingResult: ProvingResult = {
      status: PROVING_STATUS.SUCCESS,
    };
    provingState.resolve(provingResult);
  }

  // Executes the base parity circuit and stores the intermediate state for the root parity circuit
  // Enqueues the root parity circuit if all inputs are available
  private async runBaseParityCircuit(provingState: ProvingState | undefined, inputs: BaseParityInputs, index: number) {
    if (!provingState?.verifyState()) {
      logger.debug('Not running base parity, state no longer valid');
      return;
    }
    const [duration, circuitOutputs] = await elapsed(async () => {
      const [parityPublicInputs, proof] = await this.prover.getBaseParityProof(inputs);
      return new RootParityInput(proof, parityPublicInputs);
    });
    logger.debug(`Simulated base parity circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'base-parity',
      duration,
      inputSize: inputs.toBuffer().length,
      outputSize: circuitOutputs.toBuffer().length,
    } satisfies CircuitSimulationStats);

    if (!provingState?.verifyState()) {
      logger.debug(`Discarding job as state no longer valid`);
      return;
    }
    provingState.setRootParityInputs(circuitOutputs, index);

    if (!provingState.areRootParityInputsReady()) {
      // not ready to run the root parity circuit yet
      return;
    }
    const rootParityInputs = new RootParityInputs(
      provingState.rootParityInput as Tuple<RootParityInput, typeof NUM_BASE_PARITY_PER_ROOT_PARITY>,
    );
    this.enqueueJob(provingState, PROVING_JOB_TYPE.ROOT_PARITY, () =>
      this.runRootParityCircuit(provingState, rootParityInputs),
    );
  }

  // Runs the root parity circuit ans stored the outputs
  // Enqueues the root rollup proof if all inputs are available
  private async runRootParityCircuit(provingState: ProvingState | undefined, inputs: RootParityInputs) {
    if (!provingState?.verifyState()) {
      logger.debug(`Not running root parity circuit as state is no longer valid`);
      return;
    }
    const [duration, circuitOutputs] = await elapsed(async () => {
      const [parityPublicInputs, proof] = await this.prover.getRootParityProof(inputs);
      return new RootParityInput(proof, parityPublicInputs);
    });
    logger.debug(`Simulated root parity circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'root-parity',
      duration,
      inputSize: inputs.toBuffer().length,
      outputSize: circuitOutputs.toBuffer().length,
    } satisfies CircuitSimulationStats);

    if (!provingState?.verifyState()) {
      logger.debug(`Discarding job as state no longer valid`);
      return;
    }
    provingState!.finalRootParityInput = circuitOutputs;
    this.checkAndExecuteRootRollup(provingState);
  }

  private checkAndExecuteRootRollup(provingState: ProvingState | undefined) {
    if (!provingState?.isReadyForRootRollup()) {
      logger.debug('Not ready for root rollup');
      return;
    }
    this.enqueueJob(provingState, PROVING_JOB_TYPE.ROOT_ROLLUP, () => this.runRootRollup(provingState));
  }

  /**
   * Stores the inputs to a merge/root circuit and enqueues the circuit if ready
   * @param provingState - The proving state being operated on
   * @param currentLevel - The level of the merge/root circuit
   * @param currentIndex - The index of the merge/root circuit
   * @param mergeInputData - The inputs to be stored
   */
  private storeAndExecuteNextMergeLevel(
    provingState: ProvingState,
    currentLevel: bigint,
    currentIndex: bigint,
    mergeInputData: [BaseOrMergeRollupPublicInputs, Proof],
  ) {
    const result = this.storeMergeInputs(provingState, currentLevel, currentIndex, mergeInputData);

    // Are we ready to execute the next circuit?
    if (!result.ready) {
      return;
    }

    if (result.mergeLevel === 0n) {
      this.checkAndExecuteRootRollup(provingState);
    } else {
      // onto the next merge level
      this.enqueueJob(provingState, PROVING_JOB_TYPE.MERGE_ROLLUP, () =>
        this.runMergeRollup(provingState, result.mergeLevel, result.indexWithinMergeLevel, result.mergeInputData),
      );
    }
  }

  /**
   * Executes the VM circuit for a public function, will enqueue the corresponding kernel if the
   * previous kernel is ready
   * @param provingState - The proving state being operated on
   * @param txIndex - The index of the transaction being proven
   * @param functionIndex - The index of the function/kernel being proven
   */
  private async executeVM(provingState: ProvingState | undefined, txIndex: number, functionIndex: number) {
    if (!provingState?.verifyState()) {
      logger.debug(`Not running VM circuit as state is no longer valid`);
      return;
    }

    const txProvingState = provingState.getTxProvingState(txIndex);
    const publicFunction = txProvingState.getPublicFunctionState(functionIndex);

    // Prove the VM if this is a kernel that requires one
    if (!KernelTypesWithoutFunctions.has(publicFunction.publicKernelRequest.type)) {
      // Just sleep for a small amount of time
      await sleep(Math.random() * 10 + 10);
      logger.debug(`Proven VM for function index ${functionIndex} of tx index ${txIndex}`);
    }

    if (!provingState?.verifyState()) {
      logger.debug(`Not continuing after VM circuit as state is no longer valid`);
      return;
    }

    const kernelRequest = txProvingState.getNextPublicKernelFromVMProof(functionIndex, makeEmptyProof());
    if (kernelRequest.code === TX_PROVING_CODE.READY) {
      if (kernelRequest.function === undefined) {
        // Should not be possible
        throw new Error(`Error occurred, public function request undefined after VM proof completed`);
      }
      logger.debug(`Enqueuing kernel from VM for tx ${txIndex}, function ${functionIndex}`);
      this.enqueueJob(provingState, PROVING_JOB_TYPE.PUBLIC_KERNEL, () =>
        this.executePublicKernel(provingState, txIndex, functionIndex),
      );
    }
  }

  /**
   * Executes the kernel circuit for a public function, will enqueue the next kernel circuit if it's VM is already proven
   * or the base rollup circuit if there are no more kernels to be proven
   * @param provingState - The proving state being operated on
   * @param txIndex - The index of the transaction being proven
   * @param functionIndex - The index of the function/kernel being proven
   */
  private async executePublicKernel(provingState: ProvingState | undefined, txIndex: number, functionIndex: number) {
    if (!provingState?.verifyState()) {
      logger.debug(`Not running public kernel circuit as state is no longer valid`);
      return;
    }

    const txProvingState = provingState.getTxProvingState(txIndex);
    const kernelRequest = txProvingState.getPublicFunctionState(functionIndex).publicKernelRequest;

    // We may need to use the public inputs produced here instead of those coming from the sequencer
    const [_, proof] =
      kernelRequest.type == PublicKernelType.TAIL
        ? await this.prover.getPublicTailProof(kernelRequest)
        : await this.prover.getPublicKernelProof(kernelRequest);

    if (!provingState?.verifyState()) {
      logger.debug(`Not continuing after public kernel circuit as state is no longer valid`);
      return;
    }

    logger.debug(`Proven ${PublicKernelType[kernelRequest.type]} at index ${functionIndex} for tx index ${txIndex}`);

    const nextKernelRequest = txProvingState.getNextPublicKernelFromKernelProof(functionIndex, proof);
    // What's the status of the next kernel?
    if (nextKernelRequest.code === TX_PROVING_CODE.NOT_READY) {
      // Must be waiting on a VM proof
      return;
    }
    if (nextKernelRequest.code === TX_PROVING_CODE.COMPLETED) {
      // We must have completed all public function proving, we now move to the base rollup
      logger.debug(`Public functions completed for tx ${txIndex} enqueueing base rollup`);
      this.enqueueJob(provingState, PROVING_JOB_TYPE.BASE_ROLLUP, () =>
        this.runBaseRollup(provingState, BigInt(txIndex), txProvingState),
      );
      return;
    }
    // There must be another kernel ready to be proven
    if (nextKernelRequest.function === undefined) {
      // Should not be possible
      throw new Error(`Error occurred, public function request undefined after kernel proof completed`);
    }
    logger.debug(`Enqueuing kernel from kernel for tx ${txIndex}, function ${functionIndex + 1}`);
    this.enqueueJob(provingState, PROVING_JOB_TYPE.PUBLIC_KERNEL, () =>
      this.executePublicKernel(provingState, txIndex, functionIndex + 1),
    );
  }

  /**
   * Process the job queue
   * Works by managing an input queue of proof requests and an active pool of proving 'jobs'
   */
  private async processJobQueue() {
    // Used for determining the current state of a proving job
    const promiseState = (p: Promise<void>) => {
      const t = {};
      return Promise.race([p, t]).then(
        v => (v === t ? 'pending' : 'fulfilled'),
        () => 'rejected',
      );
    };

    // Just a short break between managing the sets of requests and active jobs
    const createSleepPromise = () =>
      sleep(SLEEP_TIME).then(_ => {
        return PROMISE_RESULT.SLEEP;
      });

    let sleepPromise = createSleepPromise();
    let promises: Promise<void>[] = [];
    while (!this.stopped) {
      // first look for more work
      if (this.jobQueue.length() && promises.length < this.maxConcurrentJobs) {
        // more work could be available
        const job = await this.jobQueue.get();
        if (job !== null) {
          // a proving job, add it to the pool of outstanding jobs
          promises.push(job.operation());
        }
        // continue adding more work
        continue;
      }

      // no more work to add, here we wait for any outstanding jobs to finish and/or sleep a little
      try {
        const ops = Promise.race(promises).then(_ => {
          return PROMISE_RESULT.OPERATIONS;
        });
        const result = await Promise.race([sleepPromise, ops]);
        if (result === PROMISE_RESULT.SLEEP) {
          // this is the sleep promise
          // we simply setup the promise again and go round the loop checking for more work
          sleepPromise = createSleepPromise();
          continue;
        }
      } catch (err) {
        // We shouldn't get here as all jobs should be wrapped in a 'safeJob' meaning they don't fail!
        logger.error(`Unexpected error in proving orchestrator ${err}`);
      }

      // one or more of the jobs completed, remove them
      const pendingPromises = [];
      for (const jobPromise of promises) {
        const state = await promiseState(jobPromise);
        if (state === 'pending') {
          pendingPromises.push(jobPromise);
        }
      }
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      promises = pendingPromises;
    }
  }
}
