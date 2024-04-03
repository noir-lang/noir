import { Body, L2Block, MerkleTreeId, type ProcessedTx, type TxEffect, toTxEffect } from '@aztec/circuit-types';
import { PROVING_STATUS, type ProvingResult, type ProvingTicket } from '@aztec/circuit-types/interfaces';
import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type AppendOnlyTreeSnapshot,
  type BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  type BaseRollupInputs,
  Fr,
  type GlobalVariables,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  type Proof,
  type RootParityInput,
  RootParityInputs,
} from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { MemoryFifo } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';
import { sleep } from '@aztec/foundation/sleep';
import { elapsed } from '@aztec/foundation/timer';
import { type SimulationProvider } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { inspect } from 'util';

import { type VerificationKeys, getVerificationKeys } from '../mocks/verification_keys.js';
import { type RollupProver } from '../prover/index.js';
import { RealRollupCircuitSimulator, type RollupSimulator } from '../simulator/rollup.js';
import {
  buildBaseRollupInput,
  createMergeRollupInputs,
  executeBaseParityCircuit,
  executeBaseRollupCircuit,
  executeMergeRollupCircuit,
  executeRootParityCircuit,
  executeRootRollupCircuit,
  getTreeSnapshot,
  validateTx,
} from './block-building-helpers.js';
import { type MergeRollupInputData, PROVING_JOB_TYPE, type ProvingJob, ProvingState } from './proving-state.js';

const logger = createDebugLogger('aztec:prover:proving-orchestrator');

/**
 * Implements an event driven proving scheduler to build the recursive proof tree. The idea being:
 * 1. Transactions are provided to the scheduler post simulation.
 * 2. Tree insertions are performed as required to generate transaction specific proofs
 * 3. Those transaction specific proofs are generated in the necessary order accounting for dependencies
 * 4. Once a transaction is proven, it will be incorporated into a merge proof
 * 5. Merge proofs are produced at each level of the tree until the root proof is produced
 *
 * The proving implementation is determined by the provided prover implementation. This could be for example a local prover or a remote prover pool.
 */

const SLEEP_TIME = 50;
const MAX_CONCURRENT_JOBS = 64;

enum PROMISE_RESULT {
  SLEEP,
  OPERATIONS,
}

/**
 * The orchestrator, managing the flow of recursive proving operations required to build the rollup proof tree.
 */
export class ProvingOrchestrator {
  private provingState: ProvingState | undefined = undefined;
  private jobQueue: MemoryFifo<ProvingJob> = new MemoryFifo<ProvingJob>();
  private simulator: RollupSimulator;
  private jobProcessPromise?: Promise<void>;
  private stopped = false;
  constructor(
    private db: MerkleTreeOperations,
    simulationProvider: SimulationProvider,
    protected vks: VerificationKeys,
    private prover: RollupProver,
    private maxConcurrentJobs = MAX_CONCURRENT_JOBS,
  ) {
    this.simulator = new RealRollupCircuitSimulator(simulationProvider);
  }

  public static new(db: MerkleTreeOperations, simulationProvider: SimulationProvider, prover: RollupProver) {
    const orchestrator = new ProvingOrchestrator(db, simulationProvider, getVerificationKeys(), prover);
    orchestrator.start();
    return Promise.resolve(orchestrator);
  }

  public start() {
    this.jobProcessPromise = this.processJobQueue();
  }

  public async stop() {
    this.stopped = true;
    this.jobQueue.cancel();
    await this.jobProcessPromise;
  }

  /**
   * Starts off a new block
   * @param numTxs - The number of real transactions in the block
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
    if (this.provingState && !this.provingState.isFinished()) {
      throw new Error("Can't start a new block until the previous block is finished");
    }
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

    //TODO:(@PhilWindle) Temporary until we figure out when to perform L1 to L2 insertions to make state consistency easier.
    await Promise.resolve();

    const promise = new Promise<ProvingResult>((resolve, reject) => {
      this.provingState = new ProvingState(
        numTxs,
        resolve,
        reject,
        globalVariables,
        l1ToL2MessagesPadded,
        baseParityInputs.length,
        emptyTx,
      );
    }).catch((reason: string) => ({ status: PROVING_STATUS.FAILURE, reason } as const));

    for (let i = 0; i < baseParityInputs.length; i++) {
      this.enqueueJob(this.provingState!.Id, PROVING_JOB_TYPE.BASE_PARITY, () =>
        this.runBaseParityCircuit(baseParityInputs[i], i, this.provingState!.Id),
      );
    }

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

    if (this.provingState.numTxs === this.provingState.transactionsReceived) {
      throw new Error(`Rollup already contains ${this.provingState.transactionsReceived} transactions`);
    }

    validateTx(tx);

    logger.info(`Received transaction :${tx.hash}`);

    // We start the transaction by enqueueing the state updates

    const txIndex = this.provingState!.addNewTx(tx);
    // we start this transaction off by performing it's tree insertions and
    await this.prepareBaseRollupInputs(BigInt(txIndex), tx, this.provingState!.globalVariables, this.provingState!.Id);

    if (this.provingState.transactionsReceived === this.provingState.numTxs) {
      // we need to pad the rollup with empty transactions
      const numPaddingTxs = this.provingState.numPaddingTxs;
      for (let i = 0; i < numPaddingTxs; i++) {
        const paddingTxIndex = this.provingState.addNewTx(this.provingState.emptyTx);
        await this.prepareBaseRollupInputs(
          BigInt(paddingTxIndex),
          this.provingState!.emptyTx,
          this.provingState!.globalVariables,
          this.provingState!.Id,
        );
      }
    }
  }

  /**
   * Enqueue a job to be scheduled
   * @param stateIdentifier - For state Id verification
   * @param jobType - The type of job to be queued
   * @param job - The actual job, returns a promise notifying of the job's completion
   */
  private enqueueJob(stateIdentifier: string, jobType: PROVING_JOB_TYPE, job: () => Promise<void>) {
    if (!this.provingState!.verifyState(stateIdentifier)) {
      logger(`Discarding job for state ID: ${stateIdentifier}`);
      return;
    }
    // We use a 'safeJob'. We don't want promise rejections in the proving pool, we want to capture the error here
    // and reject the proving job whilst keeping the event loop free of rejections
    const safeJob = async () => {
      try {
        await job();
      } catch (err) {
        logger.error(`Error thrown when proving job type ${PROVING_JOB_TYPE[jobType]}: ${err}`);
        this.provingState!.reject(`${err}`, stateIdentifier);
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
    index: bigint,
    tx: ProcessedTx,
    globalVariables: GlobalVariables,
    stateIdentifier: string,
  ) {
    const inputs = await buildBaseRollupInput(tx, globalVariables, this.db);
    const promises = [MerkleTreeId.NOTE_HASH_TREE, MerkleTreeId.NULLIFIER_TREE, MerkleTreeId.PUBLIC_DATA_TREE].map(
      async (id: MerkleTreeId) => {
        return { key: id, value: await getTreeSnapshot(id, this.db) };
      },
    );
    const treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot> = new Map(
      (await Promise.all(promises)).map(obj => [obj.key, obj.value]),
    );

    if (!this.provingState?.verifyState(stateIdentifier)) {
      logger(`Discarding job for state ID: ${stateIdentifier}`);
      return;
    }

    this.enqueueJob(stateIdentifier, PROVING_JOB_TYPE.BASE_ROLLUP, () =>
      this.runBaseRollup(index, tx, inputs, treeSnapshots, stateIdentifier),
    );
  }

  // Stores the intermediate inputs prepared for a merge proof
  private storeMergeInputs(
    currentLevel: bigint,
    currentIndex: bigint,
    mergeInputs: [BaseOrMergeRollupPublicInputs, Proof],
  ) {
    const mergeLevel = currentLevel - 1n;
    const indexWithinMergeLevel = currentIndex >> 1n;
    const mergeIndex = 2n ** mergeLevel - 1n + indexWithinMergeLevel;
    const subscript = Number(mergeIndex);
    const indexWithinMerge = Number(currentIndex & 1n);
    const ready = this.provingState!.storeMergeInputs(mergeInputs, indexWithinMerge, subscript);
    return { ready, indexWithinMergeLevel, mergeLevel, mergeInputData: this.provingState!.getMergeInputs(subscript) };
  }

  // Executes the base rollup circuit and stored the output as intermediate state for the parent merge/root circuit
  // Executes the next level of merge if all inputs are available
  private async runBaseRollup(
    index: bigint,
    tx: ProcessedTx,
    inputs: BaseRollupInputs,
    treeSnapshots: Map<MerkleTreeId, AppendOnlyTreeSnapshot>,
    stateIdentifier: string,
  ) {
    const [duration, baseRollupOutputs] = await elapsed(() =>
      executeBaseRollupCircuit(tx, inputs, treeSnapshots, this.simulator, this.prover, logger),
    );
    logger.debug(`Simulated base rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'base-rollup',
      duration,
      inputSize: inputs.toBuffer().length,
      outputSize: baseRollupOutputs[0].toBuffer().length,
    } satisfies CircuitSimulationStats);
    if (!this.provingState?.verifyState(stateIdentifier)) {
      logger(`Discarding job for state ID: ${stateIdentifier}`);
      return;
    }
    const currentLevel = this.provingState!.numMergeLevels + 1n;
    logger.info(`Completed base rollup at index ${index}, current level ${currentLevel}`);
    this.storeAndExecuteNextMergeLevel(currentLevel, index, baseRollupOutputs, stateIdentifier);
  }

  // Executes the merge rollup circuit and stored the output as intermediate state for the parent merge/root circuit
  // Executes the next level of merge if all inputs are available
  private async runMergeRollup(
    level: bigint,
    index: bigint,
    mergeInputData: MergeRollupInputData,
    stateIdentifier: string,
  ) {
    const circuitInputs = createMergeRollupInputs(
      [mergeInputData.inputs[0]!, mergeInputData.proofs[0]!],
      [mergeInputData.inputs[1]!, mergeInputData.proofs[1]!],
    );
    const [duration, circuitOutputs] = await elapsed(() =>
      executeMergeRollupCircuit(circuitInputs, this.simulator, this.prover, logger),
    );
    logger.debug(`Simulated merge rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'merge-rollup',
      duration,
      inputSize: circuitInputs.toBuffer().length,
      outputSize: circuitOutputs[0].toBuffer().length,
    } satisfies CircuitSimulationStats);
    if (!this.provingState?.verifyState(stateIdentifier)) {
      logger(`Discarding job for state ID: ${stateIdentifier}`);
      return;
    }
    logger.info(`Completed merge rollup at level ${level}, index ${index}`);
    this.storeAndExecuteNextMergeLevel(level, index, circuitOutputs, stateIdentifier);
  }

  // Executes the root rollup circuit
  private async runRootRollup(
    mergeInputData: MergeRollupInputData,
    rootParityInput: RootParityInput,
    stateIdentifier: string,
  ) {
    const [circuitsOutput, proof] = await executeRootRollupCircuit(
      [mergeInputData.inputs[0]!, mergeInputData.proofs[0]!],
      [mergeInputData.inputs[1]!, mergeInputData.proofs[1]!],
      rootParityInput,
      this.provingState!.newL1ToL2Messages,
      this.simulator,
      this.prover,
      this.db,
      logger,
    );
    logger.info(`Completed root rollup`);
    // Collect all new nullifiers, commitments, and contracts from all txs in this block
    const nonEmptyTxEffects: TxEffect[] = this.provingState!.allTxs.map(tx => toTxEffect(tx)).filter(
      txEffect => !txEffect.isEmpty(),
    );
    const blockBody = new Body(nonEmptyTxEffects);

    const l2Block = L2Block.fromFields({
      archive: circuitsOutput.archive,
      header: circuitsOutput.header,
      body: blockBody,
    });

    if (!l2Block.body.getTxsEffectsHash().equals(circuitsOutput.header.contentCommitment.txsEffectsHash)) {
      logger(inspect(blockBody));
      throw new Error(
        `Txs effects hash mismatch, ${l2Block.body
          .getTxsEffectsHash()
          .toString('hex')} == ${circuitsOutput.header.contentCommitment.txsEffectsHash.toString('hex')} `,
      );
    }

    const provingResult: ProvingResult = {
      status: PROVING_STATUS.SUCCESS,
      block: l2Block,
      proof,
    };
    logger.info(`Successfully proven block ${l2Block.number}!`);
    this.provingState!.resolve(provingResult, stateIdentifier);
  }

  // Executes the base parity circuit and stores the intermediate state for the root parity circuit
  // Enqueues the root parity circuit if all inputs are available
  private async runBaseParityCircuit(inputs: BaseParityInputs, index: number, stateIdentifier: string) {
    const [duration, circuitOutputs] = await elapsed(() =>
      executeBaseParityCircuit(inputs, this.simulator, this.prover, logger),
    );
    logger.debug(`Simulated base parity circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'base-parity',
      duration,
      inputSize: inputs.toBuffer().length,
      outputSize: circuitOutputs.toBuffer().length,
    } satisfies CircuitSimulationStats);
    if (!this.provingState?.verifyState(stateIdentifier)) {
      logger(`Discarding job for state ID: ${stateIdentifier}`);
      return;
    }
    this.provingState!.setRootParityInputs(circuitOutputs, index);

    if (!this.provingState!.areRootParityInputsReady()) {
      // not ready to run the root parity circuit yet
      return;
    }
    const rootParityInputs = new RootParityInputs(
      this.provingState!.rootParityInput as Tuple<RootParityInput, typeof NUM_BASE_PARITY_PER_ROOT_PARITY>,
    );
    this.enqueueJob(stateIdentifier, PROVING_JOB_TYPE.ROOT_PARITY, () =>
      this.runRootParityCircuit(rootParityInputs, stateIdentifier),
    );
  }

  // Runs the root parity circuit ans stored the outputs
  // Enqueues the root rollup proof if all inputs are available
  private async runRootParityCircuit(inputs: RootParityInputs, stateIdentifier: string) {
    const [duration, circuitOutputs] = await elapsed(() =>
      executeRootParityCircuit(inputs, this.simulator, this.prover, logger),
    );
    logger.debug(`Simulated root parity circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'root-parity',
      duration,
      inputSize: inputs.toBuffer().length,
      outputSize: circuitOutputs.toBuffer().length,
    } satisfies CircuitSimulationStats);
    if (!this.provingState?.verifyState(stateIdentifier)) {
      logger(`Discarding job for state ID: ${stateIdentifier}`);
      return;
    }
    this.provingState!.finalRootParityInput = circuitOutputs;
    this.checkAndExecuteRootRollup(stateIdentifier);
  }

  private checkAndExecuteRootRollup(stateIdentifier: string) {
    if (!this.provingState!.isReadyForRootRollup()) {
      logger('Not ready for root');
      return;
    }
    this.enqueueJob(stateIdentifier, PROVING_JOB_TYPE.ROOT_ROLLUP, () =>
      this.runRootRollup(
        this.provingState!.getMergeInputs(0)!,
        this.provingState!.finalRootParityInput!,
        stateIdentifier,
      ),
    );
  }

  private storeAndExecuteNextMergeLevel(
    currentLevel: bigint,
    currentIndex: bigint,
    mergeInputData: [BaseOrMergeRollupPublicInputs, Proof],
    stateIdentifier: string,
  ) {
    const result = this.storeMergeInputs(currentLevel, currentIndex, mergeInputData);

    // Are we ready to execute the next circuit?
    if (!result.ready) {
      return;
    }

    if (result.mergeLevel === 0n) {
      this.checkAndExecuteRootRollup(stateIdentifier);
    } else {
      // onto the next merge level
      this.enqueueJob(stateIdentifier, PROVING_JOB_TYPE.MERGE_ROLLUP, () =>
        this.runMergeRollup(result.mergeLevel, result.indexWithinMergeLevel, result.mergeInputData, stateIdentifier),
      );
    }
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
