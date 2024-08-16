import {
  type L1ToL2MessageSource,
  type L2BlockSource,
  type MerkleTreeOperations,
  type ProverClient,
  type TxProvider,
} from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { type L1Publisher } from '@aztec/sequencer-client';
import { PublicProcessorFactory, type SimulationProvider } from '@aztec/simulator';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type WorldStateSynchronizer } from '@aztec/world-state';

import { BlockProvingJob, type BlockProvingJobState } from './job/block-proving-job.js';

/**
 * An Aztec Prover Node is a standalone process that monitors the unfinalised chain on L1 for unproven blocks,
 * fetches their txs from a tx source in the p2p network or an external node, re-executes their public functions,
 * creates a rollup proof, and submits it to L1.
 */
export class ProverNode {
  private log = createDebugLogger('aztec:prover-node');
  private runningPromise: RunningPromise | undefined;
  private latestBlockWeAreProving: number | undefined;
  private jobs: Map<string, BlockProvingJob> = new Map();
  private options: { pollingIntervalMs: number; disableAutomaticProving: boolean; maxPendingJobs: number };

  constructor(
    private prover: ProverClient,
    private publisher: L1Publisher,
    private l2BlockSource: L2BlockSource,
    private l1ToL2MessageSource: L1ToL2MessageSource,
    private contractDataSource: ContractDataSource,
    private worldState: WorldStateSynchronizer,
    private txProvider: TxProvider,
    private simulator: SimulationProvider,
    private telemetryClient: TelemetryClient,
    options: { pollingIntervalMs?: number; disableAutomaticProving?: boolean; maxPendingJobs?: number } = {},
  ) {
    this.options = {
      pollingIntervalMs: 1_000,
      disableAutomaticProving: false,
      maxPendingJobs: 100,
      ...options,
    };
  }

  /**
   * Starts the prover node so it periodically checks for unproven blocks in the unfinalised chain from L1 and proves them.
   * This may change once we implement a prover coordination mechanism.
   */
  start() {
    this.runningPromise = new RunningPromise(this.work.bind(this), this.options.pollingIntervalMs);
    this.runningPromise.start();
    this.log.info('Started ProverNode');
  }

  /**
   * Stops the prover node and all its dependencies.
   */
  async stop() {
    this.log.info('Stopping ProverNode');
    await this.runningPromise?.stop();
    await this.prover.stop();
    await this.l2BlockSource.stop();
    this.publisher.interrupt();
    this.jobs.forEach(job => job.stop());
    this.log.info('Stopped ProverNode');
  }

  /**
   * Single iteration of recurring work. This method is called periodically by the running promise.
   * Checks whether there are new blocks to prove, proves them, and submits them.
   */
  protected async work() {
    try {
      if (this.options.disableAutomaticProving) {
        return;
      }

      if (!this.checkMaximumPendingJobs()) {
        this.log.debug(`Maximum pending proving jobs reached. Skipping work.`, {
          maxPendingJobs: this.options.maxPendingJobs,
          pendingJobs: this.jobs.size,
        });
        return;
      }

      const [latestBlockNumber, latestProvenBlockNumber] = await Promise.all([
        this.l2BlockSource.getBlockNumber(),
        this.l2BlockSource.getProvenBlockNumber(),
      ]);

      // Consider both the latest block we are proving and the last block proven on the chain
      const latestBlockBeingProven = this.latestBlockWeAreProving ?? 0;
      const latestProven = Math.max(latestBlockBeingProven, latestProvenBlockNumber);
      if (latestProven >= latestBlockNumber) {
        this.log.debug(`No new blocks to prove`, {
          latestBlockNumber,
          latestProvenBlockNumber,
          latestBlockBeingProven,
        });
        return;
      }

      const fromBlock = latestProven + 1;
      const toBlock = fromBlock; // We only prove one block at a time for now

      try {
        await this.startProof(fromBlock, toBlock);
      } finally {
        // If we fail to create a proving job for the given block, skip it instead of getting stuck on it.
        this.log.verbose(`Setting ${toBlock} as latest block we are proving`);
        this.latestBlockWeAreProving = toBlock;
      }
    } catch (err) {
      this.log.error(`Error in prover node work`, err);
    }
  }

  /**
   * Creates a proof for a block range. Returns once the proof has been submitted to L1.
   */
  public async prove(fromBlock: number, toBlock: number) {
    const job = await this.createProvingJob(fromBlock);
    return job.run(fromBlock, toBlock);
  }

  /**
   * Starts a proving process and returns immediately.
   */
  public async startProof(fromBlock: number, toBlock: number) {
    const job = await this.createProvingJob(fromBlock);
    void job.run(fromBlock, toBlock);
  }

  /**
   * Returns the prover instance.
   */
  public getProver() {
    return this.prover;
  }

  /**
   * Returns an array of jobs being processed.
   */
  public getJobs(): { uuid: string; status: BlockProvingJobState }[] {
    return Array.from(this.jobs.entries()).map(([uuid, job]) => ({ uuid, status: job.getState() }));
  }

  private checkMaximumPendingJobs() {
    const { maxPendingJobs } = this.options;
    return maxPendingJobs === 0 || this.jobs.size < maxPendingJobs;
  }

  private async createProvingJob(fromBlock: number) {
    if (!this.checkMaximumPendingJobs()) {
      throw new Error(`Maximum pending proving jobs ${this.options.maxPendingJobs} reached. Cannot create new job.`);
    }

    if ((await this.worldState.status()).syncedToL2Block >= fromBlock) {
      throw new Error(`Cannot create proving job for block ${fromBlock} as it is behind the current world state`);
    }

    // Fast forward world state to right before the target block and get a fork
    this.log.verbose(`Creating proving job for block ${fromBlock}`);
    const db = await this.worldState.syncImmediateAndFork(fromBlock - 1, true);

    // Create a processor using the forked world state
    const publicProcessorFactory = new PublicProcessorFactory(
      db,
      this.contractDataSource,
      this.simulator,
      this.telemetryClient,
    );

    const cleanUp = async () => {
      await db.delete();
      this.jobs.delete(job.getId());
    };

    const job = this.doCreateBlockProvingJob(db, publicProcessorFactory, cleanUp);
    this.jobs.set(job.getId(), job);
    return job;
  }

  /** Extracted for testing purposes. */
  protected doCreateBlockProvingJob(
    db: MerkleTreeOperations,
    publicProcessorFactory: PublicProcessorFactory,
    cleanUp: () => Promise<void>,
  ) {
    return new BlockProvingJob(
      this.prover.createBlockProver(db),
      publicProcessorFactory,
      this.publisher,
      this.l2BlockSource,
      this.l1ToL2MessageSource,
      this.txProvider,
      cleanUp,
    );
  }
}
